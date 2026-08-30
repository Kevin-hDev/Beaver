use super::qwen::{QwenEndpointMode, QwenRegion};

pub(super) fn base_url(
    region: QwenRegion,
    mode: QwenEndpointMode,
    workspace: Option<&str>,
) -> Option<String> {
    let value = match (mode, region) {
        (QwenEndpointMode::Shared, QwenRegion::Beijing) => {
            "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()
        }
        (QwenEndpointMode::Shared, QwenRegion::Singapore) => {
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1".to_string()
        }
        (QwenEndpointMode::Shared, QwenRegion::Virginia) => {
            "https://dashscope-us.aliyuncs.com/compatible-mode/v1".to_string()
        }
        (QwenEndpointMode::Shared, QwenRegion::HongKong) => {
            "https://cn-hongkong.dashscope.aliyuncs.com/compatible-mode/v1".to_string()
        }
        (QwenEndpointMode::Workspace, region) => {
            let workspace = workspace?;
            let host = match region {
                QwenRegion::Beijing => "cn-beijing",
                QwenRegion::Singapore => "ap-southeast-1",
                QwenRegion::Tokyo => "ap-northeast-1",
                QwenRegion::Frankfurt => "eu-central-1",
                QwenRegion::Virginia => "us-east-1",
                QwenRegion::HongKong => "cn-hongkong",
            };
            format!("https://{workspace}.{host}.maas.aliyuncs.com/compatible-mode/v1")
        }
        (QwenEndpointMode::Trial, QwenRegion::Beijing) => {
            "https://trial.cn-beijing.maas.aliyuncs.com/compatible-mode/v1".to_string()
        }
        (QwenEndpointMode::Trial, QwenRegion::Singapore) => {
            "https://trial.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1".to_string()
        }
        (QwenEndpointMode::Trial, QwenRegion::HongKong) => {
            "https://trial.cn-hongkong.maas.aliyuncs.com/compatible-mode/v1".to_string()
        }
        _ => return None,
    };
    Some(value)
}
