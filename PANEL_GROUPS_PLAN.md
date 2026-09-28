# Panel Groups — Implementation Plan

Status: planning. No code written yet. This document is the reference for the
implementation; keep it up to date as the design evolves.

## 1. Summary

Add hierarchical **panel groups**. A group is a "mini-screen": a rectangle in
its parent's space that renders its children into an offscreen target, and is
then composited into the parent as if it were a single panel that produced the
same colors and alphas.

Key properties (all confirmed with the maintainer):

- A group **clips** its children to its rectangle.
- `px` units are universal (1 px = 1 screen pixel). `w`/`h` units resolve to the
  containing group's width/height.
- Every element stays a **rectangle** (no shear), so every leaf has a
  well-defined `aspect_ratio`.
- Groups render to a target the size of their **AABB** (axis-aligned bounding
  box) at final on-screen resolution and are inserted **1:1** (no resampling),
  clipping away pixels outside the group's rotated rectangle.
- The result of a group is equivalent to a single panel with the same colors and
  alphas; the group itself has its own **blend** into the parent.
- Group backgrounds are cleared to the group's `background` color/alpha, which
  is equivalent to a full-screen color panel using `replace`.
- Ordering is **local to each group** (siblings ordered by `order`, ties broken
  by config order).

## 2. Confirmed decisions

1. Fix the `normal` blend: vulkano's `AttachmentBlend::alpha()` uses
   `SrcAlpha/OneMinusSrcAlpha` for the alpha channel too, giving
   `src_a² + dst_a(1−src_a)`. For straight-alpha accumulation we need color
   `SrcAlpha/OneMinusSrcAlpha` and alpha `One/OneMinusSrcAlpha`.
2. Composition model: resolve sizes/positions against the parent's **absolute
   pixel dimensions**; compose only the parent's **rigid** transform
   (rotation/translation). The parent's non-uniform scale never becomes a linear
   factor on a child, so rectangles never shear.
3. Clipping is done by **per-fragment discard** in the group composite shader
   (the group rectangle can be rotated, so a scissor cannot express it).
4. Group targets use the **same format/color space as the swapchain** so a group
   looks identical to the equivalent single panel.
5. Window resize recreates all group targets together with the screen buffer.
6. Pipelines are created **per group render pass** for now (no attempt to share
   compatible pipelines across render passes yet). The task graph is
   experimental; the authors recommend it.
7. Groups are always inline. **No templates / named groups / reuse.** A group is
   just an element in the tree. Named *transforms* and *materials* still exist
   and become reusable definitions.
8. The root (`panels`) is an element like any other: it can be a group or a
   single panel.
9. A transform is a definition of how a rectangle is positioned inside another
   rectangle.
10. AABB origins are integer pixel coordinates and the AABB fully covers the
    rectangle.

## 3. Config schema

`panels` changes from a list of panels to the **root element**.

```jsonc
{
  "audio": { ... },
  "transforms": { "fullscreen": { ... }, "left half": { ... } },
  "images": { "logo": "./logo.png" },
  "materials": { "foo": { ... }, "bar": { ... } },

  "panels": {
    "transform": "fullscreen",
    "order": 0,
    "blend": "normal",
    "background": [0.0, 0.0, 0.0, 0.0],
    "children": [
      { "transform": "fullscreen", "order": 0, "blend": "normal",
        "material": "foo" },
      { "transform": "left half", "order": 0, "blend": "normal",
        "background": [0.0, 0.0, 0.0, 0.0],
        "children": [
          { "transform": "right half", "order": 0, "blend": "normal",
            "material": "bar" },
        ],
      },
    ],
  },
}
```

Rust shape (in `config.rs`):

```rust
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ElementConfig {
    Panel(PanelConfig), // has `material`
    Group(GroupConfig), // has `children`
}

#[derive(Deserialize)]
pub struct PanelConfig {
    pub transform: TransformRef, // named or inline
    pub order: u32,
    pub blend: BlendConfig,
    pub material: MaterialRef,   // named or inline
}

#[derive(Deserialize)]
pub struct GroupConfig {
    pub transform: TransformRef,
    pub order: u32,
    pub blend: BlendConfig,
    #[serde(default = "default_background")]
    pub background: Vec4,        // default [0,0,0,0]
    pub children: Vec<ElementConfig>,
}
```

`panels` is just a single `ElementConfig` — there is **no** config-level root
special case, and no array form (backward compatibility is not a concern):

```rust
pub struct Config {
    pub audio: AudioSettings,
    pub transforms: HashMap<String, TransformConfig>,
    pub images: HashMap<String, String>,
    pub materials: HashMap<String, MaterialConfig>,
    pub panels: ElementConfig,
    pub background_color: Vec4, // screen clear, default [0,0,0,0]
    ...
}
```

- The runtime wraps `panels` in an implicit **screen group**:
  target = swapchain, `transform` = fullscreen identity, `order` = 0,
  `blend` = `normal`, `background` = `background_color`, children = the root
  element. The root element is therefore an ordinary element (group or panel) and
  is treated identically to every other element.
- Validation: `material` and `children` are mutually exclusive (enforced by the
  untagged split); unknown names error as today.

Existing examples (`informative.jsonc`, `waveform.jsonc`) must be migrated to
the new shape.

## 4. Runtime scene model

`scene_data.rs`:

```rust
pub enum Element {
    Panel(Panel),
    Group(Group),
}

pub struct Panel {
    pub transform: Transform,     // definition
    pub material: MaterialId,
    pub order: u32,
    pub blend: AttachmentBlend,
}

pub struct Group {
    pub transform: Transform,     // definition
    pub background: Vec4,
    pub order: u32,
    pub blend: AttachmentBlend,
    pub children: Vec<Element>,
}
```

`SceneData` keeps the tree, plus the existing `shaders`, `transforms` (named
definitions), `images`, `materials`, and `background_color`.

Named transforms/materials are resolved into the tree (`Transform` definition
cloned, material id). Groups are always inline, so no name index is needed for
them.

`MaterialId` is an index into `SceneData.materials`, as today.

## 5. Geometry & transforms

This section is the core of the change. Everything is in **pixels**.

### 5.1 Scalar/Vector resolution

Already implemented (`transform.rs`): `Scalar { value, unit }` with
`Unit::{Pixels, ScreenWidth, ScreenHeight}`, and `Vector { x, y }`. Resolving a
`Vector` against a reference size `(W, H)` in pixels gives a pixel `Vec2`:

```
pixels(x) = match unit { Pixels => value, ScreenWidth => value*W, ScreenHeight => value*H }
```

This is used for both positions and sizes, so:

```
size   = ( scale.x.pixels(parent_size),  scale.y.pixels(parent_size) )
anchor = ( position.x.pixels(parent_size), position.y.pixels(parent_size) )
```

### 5.2 Element matrix (unit quad -> parent-local pixels)

For an element with resolved size `(W,H)`, anchor position `(ax,ay)`, anchor type
`a ∈ [0,1]²`, rotation `θ`:

```
L        = S(W,H) · T(0.5,0.5) · S(0.5,0.5)     // unit quad [-1,1]² -> [0,W]x[0,H]
anchor_l = (a.x*W, a.y*H)
M_px     = T(ax,ay) · R(θ) · T(-anchor_l) · L    // unit quad -> parent-local pixels
```

`M_px` is the direct pixel-space analogue of the current `Transform::get_matrix`
(which produces NDC given a reference resolution). Refactor the math to a
pixel-space form and derive the NDC form from it.

### 5.3 Absolute geometry (top-down)

The screen is the root parent: `screen_size = swapchain extent`,
`local_to_abs(screen) = I`.

For each element under a parent with `local_to_abs` `A_p` and absolute size
`(Wp,Hp)`:

```
(Wc,Hc)  = element size resolved against (Wp,Hp)
M_px     = element matrix resolved against (Wp,Hp)      // unit quad -> parent-local
abs      = A_p · M_px                                    // unit quad -> absolute pixels
```

- Leaf: `abs` is the final absolute transform.
- Group: `A_g = abs · L(Wc,Hc)^-1` is the group's group-local->absolute matrix
  (children resolve against `(Wc,Hc)` and use `A_g` as their parent).
- `aspect_ratio` for a leaf = `Wc / Hc`.

For rendering into a target with origin `o` and size `(Wt,Ht)`:

```
to_ndc = T(-1,-1) · S(2/Wt, 2/Ht) · T(-o)
ndc    = to_ndc · abs
```

The transform buffer stores `ndc` (mat3) and `aspect_ratio`, exactly as today's
`shaders::Transform`.

Because a resolved `abs` is `T·R·S` with axis-aligned `S`, every element is a
rectangle, and its `aspect_ratio` is well-defined.

### 5.4 AABB

For a group, transform its 4 rectangle corners by `A_g` to get absolute corners.
Then:

```
origin = floor(componentwise min(corners))
max    = ceil(componentwise max(corners))
size   = max - origin            // >= 1 in each component
```

The group target extent = `size`, origin (in absolute pixels) = `origin`.

## 6. Blending & alpha

Group targets accumulate **straight alpha**. Fix the `normal` preset in
`config.rs`:

```rust
const OVER: AttachmentBlend = blend(
    // color
    BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha, BlendOp::Add,
    // alpha
    BlendFactor::One,      BlendFactor::OneMinusSrcAlpha, BlendOp::Add,
);
```

`"normal" => OVER`. The other presets are already consistent with straight
alpha (multiply multiplies color and alpha; replace replaces both; min/max/min
per channel; ignore-source leaves dst). This fix does not change current visible
output (the swapchain is opaque); it matters once group targets accumulate
alpha.

The group composite uses the group element's own blend, with `src =` the group
target's `(color, alpha)` and `dst =` the parent target. Because the buffer is
straight alpha and the background is a `replace` full-screen layer, this matches
"the group is a single panel with those colors and alphas".

## 7. Rendering algorithm (a group desugars into a render pass + a panel)

A group is desugared into two things:

1. A render pass that renders the group's children into the group's AABB target,
   with the group's rotation baked in at final resolution.
2. An ordinary **panel** in the parent that samples that target and reproduces the
   group's pixels exactly (1:1), drawn through the regular panel path (transform
   buffer, material buffer, pipeline, blend, push constants).

There is therefore no bespoke "compositing" code path. After desugaring, a
parent's children are just panels — real leaves plus one synthetic panel per
child group — and the existing draw loop handles all of them uniformly. The only
group-specific piece is the synthetic panel's internal material (sample + clip).

Post-order:

1. Render every group's children into its target. A real panel child is drawn
   with its shader/transform; a child group has already been rendered and appears
   as a synthetic panel.
2. The screen group renders into the swapchain.

Within a group, its children (real panels and synthetic panels for child groups)
are drawn in `order`.

The synthetic panel's geometry is the child group's AABB (axis-aligned), placed
1:1 in the parent target, and its material discards fragments outside the group's
rotated rectangle. Because the target was rasterized at final resolution with
the rotation baked in, the 1:1 draw reproduces the group's pixels without
resampling; the clip makes the panel occupy exactly the group's rectangle.

Clipping of a group's children is thus deferred to the synthetic panel's clip at
the parent level. AABB corners are only cleared and never survive the clip, so a
plain clear of the whole target to `background` is correct.

## 8. Group parameters (the synthetic panel's material)

The synthetic panel is an ordinary panel with an internal `GroupParameters`
material, so the transform buffer / material buffer / pipeline / push-constant
machinery is reused unchanged. The parameters are written once the target's
sampled image id is known:

```glsl
struct GroupParams {
    SampledImageId image;
    mat3 uv_to_local; // the panel's UV in [0,1]² -> group-local pixels
    vec2 size;        // group rectangle size (Wc,Hc)
};
```

Composite fragment (sample after the clip so discarded texels are never read):

```
vec2 local = (PARAMS.uv_to_local * vec3(UV, 1.0)).xy;
if (any(lessThan(local, vec2(0.0))) || any(greaterThan(local, PARAMS.size))) discard;
COLOR = texture(vko_sampler2D(PARAMS.image, sampler_id), UV);
```

- `uv_to_local` is derived from the group's inverse absolute transform and the
  AABB placement; it is constant per group and computed on the CPU.
- The synthetic panel's transform is the axis-aligned AABB placement (1:1);
  `aspect_ratio` is unused by this shader.
- Use a clamp-to-edge (and ideally nearest) sampler for group composites to avoid
  edge bleed; keep the existing repeat/linear sampler for image assets.

`GroupParameters` is an internal `TypedParameters` type (like
`ImageParameters`), not read from config. Its `SampledImageId` field is resolved
after the targets are created, exactly like `ImageParameters` resolves against
`ImageIds`.

## 9. Task graph construction

Existing: `write -> dft -> analysis`.

Added: one render node per group (including the implicit screen group). A child
group appears in its parent's node as an ordinary synthetic panel draw.

For each group `g` (post-order):

- Target: screen group -> swapchain; otherwise a *virtual* image `target_g`
  (`COLOR_ATTACHMENT | SAMPLED`, swapchain format).
- Create a virtual framebuffer `fb_g`.
- `render_g` node:
  - `.framebuffer(fb_g)`
  - `.color_attachment(target_g, COLOR_ATTACHMENT_WRITE | COLOR_ATTACHMENT_READ,
     Optimal, clear = background)`
  - `.buffer_access(...)` for the buffers its direct children read
    (global/waveform/dft/bands, its materials — including the synthetic panels'
    `GroupParams` — its transforms, its images) — the same access set as the
    current render node but scoped to the group.
  - `.image_access(target_c, FRAGMENT_SHADER_SAMPLED_READ, Optimal)` for each
    direct child group `c` (sampled by the synthetic panel).
- Edges: `analysis -> render_g` for groups with direct real-panel children;
  `render_c -> render_g` for each direct child group `c`.
- The screen group's node is the present node.

The graph is compiled once. `create_render_data` is called per group node with
that node's subpass.

## 10. Pipelines

Per group subpass, create:
- one graphics pipeline per `(leaf shader, leaf blend)` used by that group's
  direct leaf children,
- one graphics pipeline per `(group composite shader, child blend)` used by that
  group's direct child groups.

Simple version: build them eagerly per group. A later optimization can share
pipelines across compatible render passes (same format/samples/subpass layout);
`bind_pipeline` in the task graph performs no compatibility validation, so we
must ensure compatibility ourselves before doing that.

`create_graphics_pipeline` already takes a `blend`; add a variant/flag for the
group composite shader or reuse it with the composite fragment shader.

## 11. Resize

On swapchain recreation:

1. Recompute the screen size and all absolute geometry (sizes, `ndc` matrices,
   `aspect_ratio`, AABBs).
2. Recreate every offscreen target image at its new AABB extent (destroy old,
   create new) and recreate the screen/swapchain.
3. Re-register the bindless sampled image for every group target (new physical
   image -> new `SampledImageId`).
4. Rewrite the `GroupParams` buffers (inverse/size/image id).
5. Rewrite all transform buffers.
6. Rebuild per-group `RenderData` (push constants contain buffer ids, which are
   stable; the `SampledImageId`s are read from `GroupParams`, so only those
   buffers and transforms need rewriting).
7. Rebuild the `resource_map` mapping `target_g -> new physical id`.

The number and structure of task-graph nodes does not change on resize, so the
graph is not recompiled (the bloom/deferred examples recreate physical images
without recompiling and the framebuffers are built from the physical images at
execute time).

## 12. Buffers & push constants

- `Buffers.transforms` becomes **per element** (one per real panel and per
  synthetic panel), because a named transform resolves differently in each
  containing group. Named transforms are definitions only.
- Add a `groups` buffer list (one `GroupParams` buffer per synthetic panel) or
  reuse the material buffer list with internal `GroupParameters`.
- Push constants can stay as they are: the synthetic panel uses
  `transform_buffer_id`, `material_buffer_id` (group params), `sampler_id`, and
  the global buffer ids. The sampled image id lives in `GroupParams`, not in push
  constants.

## 13. File-by-file changes

- `src/config.rs`
  - `ElementConfig` / `PanelConfig` / `GroupConfig`.
  - `panels: ElementConfig`, `Vec4` background, `background_color`.
  - Fix `normal` -> `OVER`.
  - Keep named/inline transform and material refs.
- `src/video/scene_data.rs`
  - `Element`/`Group`/`Panel` tree; resolve materials per element.
  - No more flat `panels`.
- `src/video/transform.rs`
  - Pixel-space `matrix_px(parent_size)`; refactor NDC derivation.
  - AABB/corner helpers.
  - `aspect_ratio` from resolved size.
- New `src/video/geometry.rs` (or a module in scene_data)
  - Implicit screen group; top-down resolution into absolute matrices, sizes,
    AABBs, target extents.
- `src/video/material_parameters.rs`
  - `GroupParameters` (internal `GroupParams`).
- `src/video/buffers.rs`
  - Per-element transforms; group params buffers.
- `src/video/shaders.rs`
  - Register `group.glsl`.
- New `src/video/shaders/group.glsl`
  - The synthetic panel's material: sample the group target, clip to its rect.
- `src/video/tasks/render_task.rs`
  - One `RenderTask` per group; draws = ordered real panels + synthetic panels;
    clear = group background.
- `src/video/tasks/create_pipeline.rs`
  - Reused for the synthetic panel (its fragment entry + blend).
- `src/video/render_context.rs`
  - Build the group tree, targets, virtual images/framebuffers, nodes, edges,
    resource map; per-group render data; resize.
- `examples/*.jsonc`
  - Migrate to the new schema.

## 14. Phasing

1. **Alpha fix** (`normal` -> `OVER`) — small, independent, safe.
2. **Geometry refactor**: pixel-space transforms, top-down absolute resolution,
   AABB computation; unit tests. No rendering change for flat scenes.
3. **Config & scene tree**: recursive elements + implicit screen group; migrate
   examples. A flat scene (screen group + panels) should render exactly as today
   (no offscreen targets).
4. **Offscreen groups (one level)**: desugar a child group into an offscreen pass
   + a synthetic panel; task-graph nodes/edges. Target the masked-pattern case.
5. **Nesting**: recurse; post-order; multiple offscreen levels.
6. **Resize**: recreate targets + re-register sampled images + rewrite group
   params/transforms.
7. **Optimizations** (later): skip the AABB pass for a single fullscreen child
   group; share pipelines across compatible render passes; downscale huge AABBs.

## 15. Testing

- Geometry unit tests: size resolution (`w`/`h`/`px`), composition (rectangles
  preserved, no shear), AABB integer covering.
- Synthetic panel clip: a group vs. an equivalent flat set of panels must match
  pixel-for-pixel, including a rotated group and a non-normal blend.
- Alpha: a 50%-alpha layer over transparent must yield alpha 0.5 in a group
  target.
- Nested groups: 2+ levels.
- Resize: group content follows `w`/`h`; `px` content keeps its size.
- Regression: existing examples still render (after migration).

## 16. Risks & open items

- The task graph is experimental; the DAG grows significantly.
- Pipeline count per group could grow; the simple version creates them per
  subpass.
- Memory: one potentially screen-sized target per group; deep nesting multiplies
  this.
- Format: group targets need alpha and must match the swapchain's color space;
  if a swapchain format has no alpha, pick a matching-alpha format and document
  the choice.
- 1:1 composite requires integer AABB origins and exact texel alignment; verify
  no half-texel drift with linear sampling (prefer clamp/nearest for synthetic
  panels).
