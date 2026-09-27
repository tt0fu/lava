(x/5) - perceived difficulty

# Adding panel groups with their own orderings, blendings, background colors and transforms (5/5)

Looks very hard to implement nicely but makes difficult configs easier to manage. A panel group can be interpreted as a separate mini-screen on which some rendering happens, and after that the result is used like it was a single panel with a shader that produced the colors. A great example is a pattern which is masked by an image. 2 panels with the pattern shader and the image shader that multiplies it can produce a masked image that is [0, 0, 0, 0] where the image is black and [(pattern colors), 1.0] where the image is white. After the group has rendered, it can be overlaid on top of other panels, making the masked pattern give way to the content underneath. Currently, adding a pattern over some content will overwrite it, and upon masking it with an image, the overwritten content will be black instead of what was under the pattern.

## Rendering

Can probably be done with automatic taskgraph generation (write -> dft -> analyze -> (panel group tree)).

## Positioning

The "w" and "h" units become relative to the size of the panel group the panel is in.

An element can either be a panel that renders a shader, or a panel group, which contains multiple elements, which position relative to the group they're in. The screen becomes a panel group in which everything is located inside.

Will also allow for making reusable templates (for example, a pattern masked by an image) which can be reused in multiple places.

# Moving away to slang from glsl (4?/5)

Vulkano slang support is still in early beta. Will have to wait until it improves.

# Adding custom shader support (6/5)

Would require having to do shader compilation and SPIR-V reflection, dynamic shader type creation, manually managing struct offsets, etc.
