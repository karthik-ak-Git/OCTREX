use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    TextGeneration,
    Vision,
    ImageInput,
    ImageOutput,
    ToolCalling,
    FunctionCalling,
    StructuredOutput,
    JsonOutput,
    Streaming,
    Embedding,
    CodeGeneration,
}
