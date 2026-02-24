//! Research Topic Events
//!
//! Dedicated event types for research plugin integration with the HFT ecosystem.
//! Follows the standardization and trait approach from hft-event-bus.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Research-specific events for alpha signal analysis and ML model management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchEvent {
    /// Signal creation and management
    SignalCreated(SignalCreatedEvent),
    SignalUpdated(SignalUpdatedEvent),
    SignalDeleted(SignalDeletedEvent),
    
    /// Analysis and computation events
    AnalysisRequested(AnalysisRequestedEvent),
    AnalysisStarted(AnalysisStartedEvent),
    AnalysisProgress(AnalysisProgressEvent),
    AnalysisCompleted(AnalysisCompletedEvent),
    AnalysisFailed(AnalysisFailedEvent),
    
    /// Feature engineering events
    FeatureExtracted(FeatureExtractedEvent),
    FeaturePipelineUpdated(FeaturePipelineUpdatedEvent),
    
    /// ML model events
    ModelTrainingStarted(ModelTrainingStartedEvent),
    ModelTrainingProgress(ModelTrainingProgressEvent),
    ModelTrainingCompleted(ModelTrainingCompletedEvent),
    ModelDeploymentRequested(ModelDeploymentRequestedEvent),
    ModelDeploymentCompleted(ModelDeploymentCompletedEvent),
    
    /// Real-time data streaming
    RealTimeDataUpdate(RealTimeDataUpdateEvent),
    VisualizationUpdate(VisualizationUpdateEvent),
    
    /// Statistical analysis events
    StatisticalTestCompleted(StatisticalTestCompletedEvent),
    CorrelationMatrixUpdated(CorrelationMatrixUpdatedEvent),
    
    /// Configuration and state events
    ResearchConfigUpdated(ResearchConfigUpdatedEvent),
    ResearchStateChanged(ResearchStateChangedEvent),
}

impl crate::Event for ResearchEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ResearchEvent::SignalCreated(_) => "signal_created",
            ResearchEvent::SignalUpdated(_) => "signal_updated",
            ResearchEvent::SignalDeleted(_) => "signal_deleted",
            ResearchEvent::AnalysisRequested(_) => "analysis_requested",
            ResearchEvent::AnalysisStarted(_) => "analysis_started",
            ResearchEvent::AnalysisProgress(_) => "analysis_progress",
            ResearchEvent::AnalysisCompleted(_) => "analysis_completed",
            ResearchEvent::AnalysisFailed(_) => "analysis_failed",
            ResearchEvent::FeatureExtracted(_) => "feature_extracted",
            ResearchEvent::FeaturePipelineUpdated(_) => "feature_pipeline_updated",
            ResearchEvent::ModelTrainingStarted(_) => "model_training_started",
            ResearchEvent::ModelTrainingProgress(_) => "model_training_progress",
            ResearchEvent::ModelTrainingCompleted(_) => "model_training_completed",
            ResearchEvent::ModelDeploymentRequested(_) => "model_deployment_requested",
            ResearchEvent::ModelDeploymentCompleted(_) => "model_deployment_completed",
            ResearchEvent::RealTimeDataUpdate(_) => "real_time_data_update",
            ResearchEvent::VisualizationUpdate(_) => "visualization_update",
            ResearchEvent::StatisticalTestCompleted(_) => "statistical_test_completed",
            ResearchEvent::CorrelationMatrixUpdated(_) => "correlation_matrix_updated",
            ResearchEvent::ResearchConfigUpdated(_) => "research_config_updated",
            ResearchEvent::ResearchStateChanged(_) => "research_state_changed",
        }
}

// ============================================================================
// Signal Management Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCreatedEvent {
    pub signal_id: Uuid,
    pub name: String,
    pub formula: String,
    pub description: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub created_by: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalUpdatedEvent {
    pub signal_id: Uuid,
    pub updates: SignalUpdate,
    pub updated_by: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalUpdate {
    pub name: Option<String>,
    pub formula: Option<String>,
    pub description: Option<String>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDeletedEvent {
    pub signal_id: Uuid,
    pub deleted_by: String,
    pub timestamp: i64,
}

// ============================================================================
// Analysis Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequestedEvent {
    pub analysis_id: Uuid,
    pub signal_id: Uuid,
    pub dataset_id: Uuid,
    pub analysis_type: AnalysisType,
    pub config: HashMap<String, serde_json::Value>,
    pub requested_by: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    InformationCoefficient,
    StatisticalTests,
    CorrelationMatrix,
    FeatureImportance,
    ModelTraining,
    Backtest,
    MonteCarlo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStartedEvent {
    pub analysis_id: Uuid,
    pub signal_id: Uuid,
    pub dataset_id: Uuid,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgressEvent {
    pub analysis_id: Uuid,
    pub progress: f64, // 0.0 to 1.0
    pub current_step: String,
    pub estimated_remaining: Option<i64>, // seconds
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCompletedEvent {
    pub analysis_id: Uuid,
    pub signal_id: Uuid,
    pub dataset_id: Uuid,
    pub results: AnalysisResults,
    pub completed_at: i64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub ic_results: Option<IcResults>,
    pub statistical_tests: Option<StatisticalTestResults>,
    pub correlation_matrix: Option<CorrelationMatrix>,
    pub feature_importance: Option<FeatureImportance>,
    pub model_metrics: Option<ModelMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcResults {
    pub mean_ic: f64,
    pub std_ic: f64,
    pub ir: f64, // Information Ratio
    pub hit_rate: f64,
    pub ic_series: Vec<(i64, f64)>, // (timestamp, ic_value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTestResults {
    pub normality_test: Option<NormalityTestResult>,
    pub stationarity_test: Option<StationarityTestResult>,
    pub autocorrelation_test: Option<AutocorrelationTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityTestResult {
    pub test_type: String, // "Shapiro-Wilk", "Jarque-Bera", "Kolmogorov-Smirnov"
    pub statistic: f64,
    pub p_value: f64,
    pub is_normal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationarityTestResult {
    pub test_type: String, // "ADF", "KPSS", "PP"
    pub statistic: f64,
    pub p_value: f64,
    pub is_stationary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocorrelationTestResult {
    pub lags: Vec<usize>,
    pub autocorrelations: Vec<f64>,
    pub confidence_intervals: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub signals: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub method: String, // "pearson", "spearman", "kendall"
    pub p_values: Option<Vec<Vec<f64>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub features: Vec<String>,
    pub importance_scores: Vec<f64>,
    pub method: String, // "gain", "permutation", "shap"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub accuracy: Option<f64>,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1_score: Option<f64>,
    pub mse: Option<f64>,
    pub rmse: Option<f64>,
    pub r_squared: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFailedEvent {
    pub analysis_id: Uuid,
    pub signal_id: Uuid,
    pub dataset_id: Uuid,
    pub error: String,
    pub failed_at: i64,
}

// ============================================================================
// Feature Engineering Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureExtractedEvent {
    pub signal_id: Uuid,
    pub timestamp: i64,
    pub features: HashMap<String, f64>,
    pub feature_names: Vec<String>,
    pub extraction_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturePipelineUpdatedEvent {
    pub pipeline_id: String,
    pub updates: Vec<PipelineUpdate>,
    pub updated_by: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineUpdate {
    pub operation: String, // "add_feature", "remove_feature", "modify_feature"
    pub feature_name: String,
    pub config: HashMap<String, serde_json::Value>,
}

// ============================================================================
// ML Model Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTrainingStartedEvent {
    pub model_id: Uuid,
    pub model_type: String,
    pub signal_id: Uuid,
    pub dataset_id: Uuid,
    pub hyperparameters: HashMap<String, serde_json::Value>,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTrainingProgressEvent {
    pub model_id: Uuid,
    pub epoch: u32,
    pub total_epochs: u32,
    pub training_loss: f64,
    pub validation_loss: Option<f64>,
    pub metrics: HashMap<String, f64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTrainingCompletedEvent {
    pub model_id: Uuid,
    pub final_metrics: ModelMetrics,
    pub training_time_ms: u64,
    pub model_path: String,
    pub completed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDeploymentRequestedEvent {
    pub model_id: Uuid,
    pub deployment_config: HashMap<String, serde_json::Value>,
    pub requested_by: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDeploymentCompletedEvent {
    pub model_id: Uuid,
    pub deployment_id: String,
    pub endpoint: String,
    pub deployed_at: i64,
}

// ============================================================================
// Real-time Data Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeDataUpdateEvent {
    pub symbol: String,
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub features: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationUpdateEvent {
    pub visualization_id: String,
    pub update_type: VisualizationUpdateType,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationUpdateType {
    ChartData,
    SignalUpdate,
    AnalysisResult,
    ModelPerformance,
    RealTimeMetrics,
}

// ============================================================================
// Statistical Analysis Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTestCompletedEvent {
    pub test_id: Uuid,
    pub test_type: String,
    pub signal_id: Uuid,
    pub results: StatisticalTestResults,
    pub completed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrixUpdatedEvent {
    pub matrix_id: Uuid,
    pub signals: Vec<String>,
    pub correlation_matrix: CorrelationMatrix,
    pub updated_at: i64,
}

// ============================================================================
// Configuration Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfigUpdatedEvent {
    pub config_section: String,
    pub updates: HashMap<String, serde_json::Value>,
    pub updated_by: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStateChangedEvent {
    pub component: String,
    pub old_state: String,
    pub new_state: String,
    pub timestamp: i64,
}

// ============================================================================
// Trait Implementation for Event Bus Integration
// ============================================================================

impl crate::Event for ResearchEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ResearchEvent::SignalCreated(_) => "signal_created",
            ResearchEvent::SignalUpdated(_) => "signal_updated",
            ResearchEvent::SignalDeleted(_) => "signal_deleted",
            ResearchEvent::AnalysisRequested(_) => "analysis_requested",
            ResearchEvent::AnalysisStarted(_) => "analysis_started",
            ResearchEvent::AnalysisProgress(_) => "analysis_progress",
            ResearchEvent::AnalysisCompleted(_) => "analysis_completed",
            ResearchEvent::AnalysisFailed(_) => "analysis_failed",
            ResearchEvent::FeatureExtracted(_) => "feature_extracted",
            ResearchEvent::FeaturePipelineUpdated(_) => "feature_pipeline_updated",
            ResearchEvent::ModelTrainingStarted(_) => "model_training_started",
            ResearchEvent::ModelTrainingProgress(_) => "model_training_progress",
            ResearchEvent::ModelTrainingCompleted(_) => "model_training_completed",
            ResearchEvent::ModelDeploymentRequested(_) => "model_deployment_requested",
            ResearchEvent::ModelDeploymentCompleted(_) => "model_deployment_completed",
            ResearchEvent::RealTimeDataUpdate(_) => "real_time_data_update",
            ResearchEvent::VisualizationUpdate(_) => "visualization_update",
            ResearchEvent::StatisticalTestCompleted(_) => "statistical_test_completed",
            ResearchEvent::CorrelationMatrixUpdated(_) => "correlation_matrix_updated",
            ResearchEvent::ResearchConfigUpdated(_) => "research_config_updated",
            ResearchEvent::ResearchStateChanged(_) => "research_state_changed",
        }
    }
}

impl crate::MarketEvent for ResearchEvent {
    fn timestamp(&self) -> i64 {
        match self {
            ResearchEvent::SignalCreated(e) => e.timestamp,
            ResearchEvent::SignalUpdated(e) => e.timestamp,
            ResearchEvent::SignalDeleted(e) => e.timestamp,
            ResearchEvent::AnalysisRequested(e) => e.timestamp,
            ResearchEvent::AnalysisStarted(e) => e.started_at,
            ResearchEvent::AnalysisProgress(e) => e.timestamp,
            ResearchEvent::AnalysisCompleted(e) => e.completed_at,
            ResearchEvent::AnalysisFailed(e) => e.failed_at,
            ResearchEvent::FeatureExtracted(e) => e.timestamp,
            ResearchEvent::FeaturePipelineUpdated(e) => e.timestamp,
            ResearchEvent::ModelTrainingStarted(e) => e.started_at,
            ResearchEvent::ModelTrainingProgress(e) => e.timestamp,
            ResearchEvent::ModelTrainingCompleted(e) => e.completed_at,
            ResearchEvent::ModelDeploymentRequested(e) => e.timestamp,
            ResearchEvent::ModelDeploymentCompleted(e) => e.deployed_at,
            ResearchEvent::RealTimeDataUpdate(e) => e.timestamp,
            ResearchEvent::VisualizationUpdate(e) => e.timestamp,
            ResearchEvent::StatisticalTestCompleted(e) => e.completed_at,
            ResearchEvent::CorrelationMatrixUpdated(e) => e.updated_at,
            ResearchEvent::ResearchConfigUpdated(e) => e.timestamp,
            ResearchEvent::ResearchStateChanged(e) => e.timestamp,
        }
    }

    fn symbol(&self) -> Option<&str> {
        match self {
            ResearchEvent::RealTimeDataUpdate(e) => Some(&e.symbol),
            _ => None,
        }
    }
    
    fn priority(&self) -> u8 {
        match self {
            ResearchEvent::RealTimeDataUpdate(_) => 1, // Highest priority for real-time
            ResearchEvent::AnalysisStarted(_) | ResearchEvent::AnalysisProgress(_) => 2,
            ResearchEvent::SignalCreated(_) | ResearchEvent::SignalUpdated(_) => 3,
            ResearchEvent::AnalysisCompleted(_) | ResearchEvent::AnalysisFailed(_) => 4,
            _ => 5, // Default priority
        }
    }
}
