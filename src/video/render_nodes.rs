use vulkano_taskgraph::{
    QueueFamilyType,
    graph::{AttachmentInfo, NodeId, TaskGraph},
    resource::{AccessTypes, ImageLayoutType},
};

use crate::video::{
    geometry::ResolvedChild,
    render_context::RenderContext,
    render_graph::RenderGraphInputs,
    tasks::{
        analysis_task::AnalysisTask, dft_task::DftTask, draw::Target, render_task::RenderTask,
        write_task::WriteTask,
    },
};

/// Adds the audio compute chain and returns the node every group render node depends on.
pub fn add_compute_nodes(
    task_graph: &mut TaskGraph<RenderContext>,
    inputs: &RenderGraphInputs<'_>,
) -> NodeId {
    let buffers = inputs.buffers;
    let queue_family = QueueFamilyType::Compute;
    let compute_push_constants = inputs.storage_buffers.compute_push_constants();

    let write_node = task_graph
        .create_task_node("Write", queue_family, WriteTask {})
        .buffer_access(buffers.global, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
        .buffer_access(buffers.waveform, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
        .build();
    let dft_node = task_graph
        .create_task_node(
            "Dft",
            queue_family,
            DftTask::new(
                inputs.audio_settings,
                inputs.device,
                inputs.bcx,
                compute_push_constants,
            ),
        )
        .buffer_access(buffers.waveform, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
        .buffer_access(
            buffers.dft,
            AccessTypes::COMPUTE_SHADER_STORAGE_READ | AccessTypes::COMPUTE_SHADER_STORAGE_WRITE,
        )
        .build();
    let analysis_node = task_graph
        .create_task_node(
            "Analysis",
            queue_family,
            AnalysisTask::new(
                inputs.audio_settings,
                inputs.device,
                inputs.bcx,
                compute_push_constants,
            ),
        )
        .buffer_access(buffers.global, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
        .buffer_access(buffers.dft, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
        .buffer_access(
            buffers.bands,
            AccessTypes::COMPUTE_SHADER_STORAGE_READ | AccessTypes::COMPUTE_SHADER_STORAGE_WRITE,
        )
        .buffer_access(
            buffers.waveform,
            AccessTypes::COMPUTE_SHADER_STORAGE_READ | AccessTypes::COMPUTE_SHADER_STORAGE_WRITE,
        )
        .build();

    task_graph.add_edge(write_node, dft_node).unwrap();
    task_graph.add_edge(dft_node, analysis_node).unwrap();
    analysis_node
}

/// Adds one render node per group, in reverse order so children are created before their parents.
pub fn add_group_nodes(
    task_graph: &mut TaskGraph<RenderContext>,
    inputs: &RenderGraphInputs<'_>,
    dependency: NodeId,
) -> Vec<NodeId> {
    let resolved = inputs.resolved;
    let buffers = inputs.buffers;
    let mut group_nodes = vec![None; resolved.groups.len()];
    for group in (0..resolved.groups.len()).rev() {
        let info = &resolved.groups[group];
        let is_root = group == resolved.root;
        let target = if is_root {
            Target::Swapchain(inputs.virtual_swapchain_id)
        } else {
            Target::Image(inputs.group_targets.virtual_target(group))
        };
        let framebuffer_id = task_graph.add_framebuffer();
        let mut node = task_graph.create_task_node(
            "Group",
            QueueFamilyType::Graphics,
            RenderTask::new(
                inputs.vertex_buffer_id,
                group,
                target,
                info.background.into(),
            ),
        );
        node.framebuffer(framebuffer_id).color_attachment(
            if is_root {
                inputs.virtual_swapchain_id.current_image_id()
            } else {
                inputs.group_targets.virtual_target(group)
            },
            AccessTypes::COLOR_ATTACHMENT_READ | AccessTypes::COLOR_ATTACHMENT_WRITE,
            ImageLayoutType::Optimal,
            &AttachmentInfo {
                clear: true,
                ..Default::default()
            },
        );
        for child in &info.children {
            let (material, transform) = match child {
                ResolvedChild::Panel(panel) => (resolved.panels[*panel].material, *panel),
                ResolvedChild::Group(child_group) => (
                    buffers.group_material(*child_group),
                    resolved.panels.len() + (child_group - 1),
                ),
            };
            node.buffer_access(
                buffers.transforms[transform],
                AccessTypes::VERTEX_SHADER_STORAGE_READ | AccessTypes::FRAGMENT_SHADER_STORAGE_READ,
            );
            node.buffer_access(
                buffers.materials[material],
                AccessTypes::FRAGMENT_SHADER_STORAGE_READ,
            );
        }
        node.buffer_access(buffers.global, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .buffer_access(buffers.waveform, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .buffer_access(buffers.dft, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .buffer_access(buffers.bands, AccessTypes::FRAGMENT_SHADER_STORAGE_READ);
        for &image_id in inputs.scene_image_ids {
            node.image_access(
                image_id,
                AccessTypes::FRAGMENT_SHADER_SAMPLED_READ,
                ImageLayoutType::Optimal,
            );
        }
        for child in &info.children {
            if let ResolvedChild::Group(child_group) = child {
                node.image_access(
                    inputs.group_targets.virtual_target(*child_group),
                    AccessTypes::FRAGMENT_SHADER_SAMPLED_READ,
                    ImageLayoutType::Optimal,
                );
            }
        }
        let node_id = node.build();
        group_nodes[group] = Some(node_id);
        task_graph.add_edge(dependency, node_id).unwrap();
        for child in &info.children {
            if let ResolvedChild::Group(child_group) = child {
                task_graph
                    .add_edge(group_nodes[*child_group].unwrap(), node_id)
                    .unwrap();
            }
        }
    }
    group_nodes.into_iter().map(Option::unwrap).collect()
}
