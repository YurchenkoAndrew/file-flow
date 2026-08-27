use fastembed::{InitOptionsUserDefined, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel};
use std::fs;
use tauri::Manager;

pub struct NeuralEmbedder {
    model: TextEmbedding,
}

impl NeuralEmbedder {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let resource_dir = app_handle
            .path()
            .resource_dir()
            .map_err(|_| "Не удалось получить путь к директории ресурсов Tauri")?;

        let model_dir = resource_dir
            .join("resources")
            .join("ai_models")
            .join("all-MiniLM-L6-v2");

        if !model_dir.exists() {
            return Err(format!("Папка с нейросетью не найдена по пути: {:?}", model_dir).into());
        }

        // Читаем файлы модели в виде байтов (Vec<u8>)
        let onnx_bytes = fs::read(model_dir.join("model.onnx"))?;
        let tokenizer_bytes = fs::read(model_dir.join("tokenizer.json"))?;
        let config_bytes = fs::read(model_dir.join("config.json"))?;
        let special_tokens_bytes = fs::read(model_dir.join("special_tokens_map.json"))?;

        // ИСправляем Option на пустой вектор по умолчанию
        let tokenizer_config_bytes =
            fs::read(model_dir.join("tokenizer_config.json")).unwrap_or_default();

        let user_model = UserDefinedEmbeddingModel {
            onnx_file: onnx_bytes,
            external_initializers: vec![], // Оставляем пустой вектор
            tokenizer_files: TokenizerFiles {
                tokenizer_file: tokenizer_bytes,
                config_file: config_bytes,
                special_tokens_map_file: special_tokens_bytes, // Передаем прочитанные байты сюда
                tokenizer_config_file: tokenizer_config_bytes,
            },
            pooling: None,
            quantization: Default::default(),
            output_key: None,
        };

        let options = InitOptionsUserDefined::default();
        let model = TextEmbedding::try_new_from_user_defined(user_model, options)?;

        Ok(Self { model })
    }

    pub fn generate_embeddings(
        &mut self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings)
    }
}
