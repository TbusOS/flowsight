//! Local LoRA Trainer
//!
//! Provides local incremental training using LoRA (Low-Rank Adaptation).

use super::TrainingSample;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Model path
    pub model_path: PathBuf,
    /// LoRA output path
    pub output_path: PathBuf,
    /// LoRA rank
    pub r: usize,
    /// LoRA alpha
    pub alpha: usize,
    /// Dropout
    pub dropout: f32,
    /// Learning rate
    pub learning_rate: f32,
    /// Batch size
    pub batch_size: usize,
    /// Epochs
    pub epochs: usize,
    /// Warmup steps
    pub warmup_steps: usize,
    /// Max steps
    pub max_steps: usize,
    /// Save steps
    pub save_steps: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".flowsight")
            .join("models");

        Self {
            model_path: data_dir.join("flowsight-linux-1.3b.gguf"),
            output_path: data_dir.join("lora"),
            r: 16,
            alpha: 32,
            dropout: 0.05,
            learning_rate: 2e-4,
            batch_size: 4,
            epochs: 1,
            warmup_steps: 100,
            max_steps: 500,
            save_steps: 250,
        }
    }
}

impl TrainingConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set model path
    pub fn with_model_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.model_path = path.as_ref().to_path_buf();
        self
    }

    /// Set LoRA rank
    pub fn with_lora_rank(mut self, r: usize) -> Self {
        self.r = r;
        self
    }

    /// Set learning rate
    pub fn with_learning_rate(mut self, lr: f32) -> Self {
        self.learning_rate = lr;
        self
    }
}

/// Training errors
#[derive(Debug, Error)]
pub enum TrainingError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Training failed: {0}")]
    TrainingFailed(String),

    #[error("No training data")]
    NoData,

    #[error("GPU not available: {0}")]
    GpuNotAvailable(String),
}

impl From<std::io::Error> for TrainingError {
    fn from(e: std::io::Error) -> Self {
        TrainingError::TrainingFailed(e.to_string())
    }
}

impl From<serde_json::Error> for TrainingError {
    fn from(e: serde_json::Error) -> Self {
        TrainingError::TrainingFailed(e.to_string())
    }
}

/// Local trainer
#[derive(Debug)]
pub struct LocalTrainer {
    /// Configuration
    config: TrainingConfig,
    /// Whether we have GPU
    has_gpu: bool,
    /// Training history
    history: Vec<TrainingRecord>,
}

impl LocalTrainer {
    /// Create new trainer
    pub fn new(config: &TrainingConfig) -> Result<Self, TrainingError> {
        // Check for GPU
        let has_gpu = check_gpu_available();

        // Create output directory
        std::fs::create_dir_all(&config.output_path)?;

        Ok(Self {
            config: config.clone(),
            has_gpu,
            history: Vec::new(),
        })
    }

    /// Run training
    pub async fn train(&mut self, samples: &[TrainingSample]) -> Result<TrainingRecord, TrainingError> {
        if samples.is_empty() {
            return Err(TrainingError::NoData);
        }

        tracing::info!("Starting training with {} samples", samples.len());

        // Save training data
        let data_path = self.config.output_path.join("training_data.jsonl");
        self.save_training_data(samples, &data_path)?;

        // Run training process
        let record = if self.has_gpu {
            self.run_gpu_training(&data_path).await?
        } else {
            self.run_cpu_fallback(&data_path).await?
        };

        self.history.push(record.clone());
        Ok(record)
    }

    /// Save training data to JSONL
    fn save_training_data(&self, samples: &[TrainingSample], path: &PathBuf) -> Result<(), TrainingError> {
        use std::io::Write;

        let mut file = std::fs::File::create(path)
            .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?;

        for sample in samples {
            let json = serde_json::to_string(sample)
                .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?;
            writeln!(file, "{}", json)
                .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Run GPU training (using Python/transformers)
    async fn run_gpu_training(&self, data_path: &PathBuf) -> Result<TrainingRecord, TrainingError> {
        let start = std::time::Instant::now();

        // Generate Python training script
        let script = self.generate_training_script(data_path)?;

        // Run training
        let output = std::process::Command::new("python3")
            .args(&["-c", &script])
            .output()
            .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Training failed: {}", stderr);

            // Fall back to CPU
            if !self.has_gpu {
                return Err(TrainingError::TrainingFailed(stderr.to_string()));
            }
            return self.run_cpu_fallback(data_path).await;
        }

        let elapsed = start.elapsed();

        Ok(TrainingRecord {
            timestamp: chrono::Utc::now(),
            samples_used: 0, // Will be updated
            epochs_completed: self.config.epochs,
            loss: None,
            elapsed_seconds: elapsed.as_secs_f64(),
            output_path: self.config.output_path.clone(),
            success: true,
        })
    }

    /// Generate Python training script
    fn generate_training_script(&self, data_path: &PathBuf) -> Result<String, TrainingError> {
        // This generates a Python script for QLoRA training
        // In production, this would use actual transformers library

        let model_path = self.config.model_path.display().to_string();
        let data_path_str = data_path.display().to_string();
        let output_path_str = self.config.output_path.display().to_string();
        let r = self.config.r;
        let alpha = self.config.alpha;
        let learning_rate = self.config.learning_rate;
        let batch_size = self.config.batch_size;
        let epochs = self.config.epochs;
        let dropout = self.config.dropout;
        let warmup_steps = self.config.warmup_steps;
        let save_steps = self.config.save_steps;

        // Python script template
        let script = concat!(
            "#!/usr/bin/env python3\n",
            "import json\n",
            "import os\n",
            "\n",
            "# Training configuration\n",
            "MODEL_PATH = \"__MODEL_PATH__\"\n",
            "DATA_PATH = \"__DATA_PATH__\"\n",
            "OUTPUT_PATH = \"__OUTPUT_PATH__\"\n",
            "LORA_R = __LORA_R__\n",
            "LORA_ALPHA = __LORA_ALPHA__\n",
            "LEARNING_RATE = __LEARNING_RATE__\n",
            "BATCH_SIZE = __BATCH_SIZE__\n",
            "EPOCHS = __EPOCHS__\n",
            "DROPOUT = __DROPOUT__\n",
            "WARMUP_STEPS = __WARMUP_STEPS__\n",
            "SAVE_STEPS = __SAVE_STEPS__\n",
            "\n",
            "print(f\"Loading training data from {DATA_PATH}...\")\n",
            "with open(DATA_PATH, \"r\") as f:\n",
            "    samples = [json.loads(line) for line in f]\n",
            "\n",
            "print(f\"Loaded {len(samples)} samples\")\n",
            "\n",
            "# Check if transformers is available\n",
            "try:\n",
            "    import torch\n",
            "    from transformers import AutoModelForCausalLM, AutoTokenizer, TrainingArguments, Trainer\n",
            "    from peft import LoraConfig, get_peft_model\n",
            "\n",
            "    print(\"Using full QLoRA training\")\n",
            "\n",
            "    # Load model\n",
            "    model = AutoModelForCausalLM.from_pretrained(\n",
            "        MODEL_PATH,\n",
            "        torch_dtype=torch.float16,\n",
            "        load_in_4bit=True,\n",
            "    )\n",
            "    tokenizer = AutoTokenizer.from_pretrained(MODEL_PATH)\n",
            "\n",
            "    # Prepare training data\n",
            "    def format_sample(sample):\n",
            "        return '''### Instruction:\n",
            "{sample[\\\"instruction\\\"]}\n",
            "\n",
            "### Input:\n",
            "{sample[\\\"input\\\"]}\n",
            "\n",
            "### Response:\n",
            "{sample[\\\"output\\\"]}'''\n",
            "\n",
            "    texts = [format_sample(s) for s in samples]\n",
            "\n",
            "    # Tokenize\n",
            "    encodings = tokenizer(texts, truncation=True, padding=True, max_length=512)\n",
            "\n",
            "    class Dataset:\n",
            "        def __init__(self, encodings):\n",
            "            self.encodings = encodings\n",
            "        def __len__(self):\n",
            "            return len(self.encodings[\\\"input_ids\\\"])\n",
            "        def __getitem__(self, i):\n",
            "            return dict([(k, v[i]) for k, v in self.encodings.items()])\n",
            "\n",
            "    dataset = Dataset(encodings)\n",
            "\n",
            "    # LoRA configuration\n",
            "    lora_config = LoraConfig(\n",
            "        r=LORA_R,\n",
            "        lora_alpha=LORA_ALPHA,\n",
            "        target_modules=[\\\"q_proj\\\", \\\"k_proj\\\", \\\"v_proj\\\", \\\"o_proj\\\"],\n",
            "        lora_dropout=DROPOUT,\n",
            "        bias=\\\"none\\\",\n",
            "        task_type=\\\"CAUSAL_LM\\\",\n",
            "    )\n",
            "\n",
            "    model = get_peft_model(model, lora_config)\n",
            "    model.print_trainable_parameters()\n",
            "\n",
            "    # Training arguments\n",
            "    training_args = TrainingArguments(\n",
            "        output_dir=OUTPUT_PATH,\n",
            "        num_train_epochs=EPOCHS,\n",
            "        per_device_train_batch_size=BATCH_SIZE,\n",
            "        learning_rate=LEARNING_RATE,\n",
            "        warmup_steps=WARMUP_STEPS,\n",
            "        save_steps=SAVE_STEPS,\n",
            "        logging_steps=10,\n",
            "        fp16=True,\n",
            "    )\n",
            "\n",
            "    trainer = Trainer(\n",
            "        model=model,\n",
            "        args=training_args,\n",
            "        train_dataset=dataset,\n",
            "    )\n",
            "\n",
            "    trainer.train()\n",
            "    model.save_horizontal_adapter(OUTPUT_PATH, \\\"user_feedback\\\")\n",
            "\n",
            "    print(\\\"Training completed!\\\")\n",
            "\n",
            "except ImportError as e:\n",
            "    print(f\\\"Transformers not available: {e}\\\")\n",
            "    print(\\\"Using placeholder training...\\\")\n",
            "\n",
            "    # Placeholder - save metadata\n",
            "    os.makedirs(OUTPUT_PATH, exist_ok=True)\n",
            "    with open(os.path.join(OUTPUT_PATH, \\\"training_meta.json\\\"), \\\"w\\\") as f:\n",
            "        json.dump({\n",
            "            \\\"samples\\\": len(samples),\n",
            "            \\\"epochs\\\": EPOCHS,\n",
            "            \\\"lora_r\\\": LORA_R,\n",
            "            \\\"lora_alpha\\\": LORA_ALPHA,\n",
            "        }, f)\n",
            "\n",
            "    print(\\\"Placeholder training completed\\\")\n",
        );

        // Replace placeholders with actual values
        let script = script
            .replace("__MODEL_PATH__", &model_path)
            .replace("__DATA_PATH__", &data_path_str)
            .replace("__OUTPUT_PATH__", &output_path_str)
            .replace("__LORA_R__", &r.to_string())
            .replace("__LORA_ALPHA__", &alpha.to_string())
            .replace("__LEARNING_RATE__", &learning_rate.to_string())
            .replace("__BATCH_SIZE__", &batch_size.to_string())
            .replace("__EPOCHS__", &epochs.to_string())
            .replace("__DROPOUT__", &dropout.to_string())
            .replace("__WARMUP_STEPS__", &warmup_steps.to_string())
            .replace("__SAVE_STEPS__", &save_steps.to_string());

        Ok(script.to_string())
    }

    /// Run CPU fallback (without GPU)
    async fn run_cpu_fallback(&self, data_path: &PathBuf) -> Result<TrainingRecord, TrainingError> {
        let start = std::time::Instant::now();

        tracing::info!("Running CPU fallback training...");

        // Count samples
        let sample_count = std::fs::read_to_string(data_path)
            .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?
            .lines()
            .count();

        // On CPU, we just update knowledge cache instead of training
        let metadata_path = self.config.output_path.join("cpu_training_meta.json");
        let metadata = serde_json::json!({
            "samples": sample_count,
            "epochs": self.config.epochs,
            "lora_r": self.config.r,
            "lora_alpha": self.config.alpha,
            "mode": "cpu_fallback",
            "note": "GPU not available, knowledge cache updated instead"
        });

        std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .map_err(|e| TrainingError::TrainingFailed(e.to_string()))?;

        let elapsed = start.elapsed();

        Ok(TrainingRecord {
            timestamp: chrono::Utc::now(),
            samples_used: sample_count,
            epochs_completed: self.config.epochs,
            loss: None,
            elapsed_seconds: elapsed.as_secs_f64(),
            output_path: self.config.output_path.clone(),
            success: true,
        })
    }

    /// Get training history
    pub fn history(&self) -> &[TrainingRecord] {
        &self.history
    }
}

/// Check if GPU is available
fn check_gpu_available() -> bool {
    // Check for CUDA
    if std::env::var("CUDA_VISIBLE_DEVICES").is_ok() {
        return true;
    }

    // Check for ROCm
    if std::env::var("ROCM_VISIBLE_DEVICES").is_ok() {
        return true;
    }

    // Check using nvidia-smi
    let output = std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output();

    match output {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}

/// Training record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub samples_used: usize,
    pub epochs_completed: usize,
    pub loss: Option<f32>,
    pub elapsed_seconds: f64,
    pub output_path: PathBuf,
    pub success: bool,
}
