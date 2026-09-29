use std::sync::Arc;

use vulkano::instance::{
    Instance,
    debug::{
        DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
        DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo,
    },
};

pub fn create_debug_messenger(instance: &Arc<Instance>) -> Option<DebugUtilsMessenger> {
    unsafe {
        DebugUtilsMessenger::new(
            instance,
            &DebugUtilsMessengerCreateInfo {
                message_severity: DebugUtilsMessageSeverity::ERROR
                    | DebugUtilsMessageSeverity::WARNING
                    | DebugUtilsMessageSeverity::INFO
                    | DebugUtilsMessageSeverity::VERBOSE,
                message_type: DebugUtilsMessageType::GENERAL
                    | DebugUtilsMessageType::VALIDATION
                    | DebugUtilsMessageType::PERFORMANCE,
                ..DebugUtilsMessengerCreateInfo::new(&DebugUtilsMessengerCallback::new(
                    |message_severity, message_type, callback_data| {
                        println!(
                            "{} {} {}: {}",
                            callback_data.message_id_name.unwrap_or("unknown"),
                            type_name(message_type),
                            severity_name(message_severity),
                            callback_data.message
                        );
                    },
                ))
            },
        )
    }
    .ok()
}

fn severity_name(severity: DebugUtilsMessageSeverity) -> &'static str {
    if severity.intersects(DebugUtilsMessageSeverity::ERROR) {
        "error"
    } else if severity.intersects(DebugUtilsMessageSeverity::WARNING) {
        "warning"
    } else if severity.intersects(DebugUtilsMessageSeverity::INFO) {
        "information"
    } else if severity.intersects(DebugUtilsMessageSeverity::VERBOSE) {
        "verbose"
    } else {
        "unknown"
    }
}

fn type_name(message_type: DebugUtilsMessageType) -> &'static str {
    if message_type.intersects(DebugUtilsMessageType::GENERAL) {
        "general"
    } else if message_type.intersects(DebugUtilsMessageType::VALIDATION) {
        "validation"
    } else if message_type.intersects(DebugUtilsMessageType::PERFORMANCE) {
        "performance"
    } else {
        "unknown"
    }
}
