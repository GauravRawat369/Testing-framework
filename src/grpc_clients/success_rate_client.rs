use error_stack::ResultExt;
use success_rate::{success_rate_calculator_client::SuccessRateCalculatorClient, CalSuccessRateConfig, CalSuccessRateRequest, CalSuccessRateResponse, InvalidateWindowsRequest, InvalidateWindowsResponse, LabelWithStatus, SuccessRateSpecificityLevel as ProtoSpecificityLevel, UpdateSuccessRateWindowConfig, UpdateSuccessRateWindowRequest, UpdateSuccessRateWindowResponse};
use super::{create_grpc_request, health_check_client::{Client, CustomResult}};
use crate::types::Key;

use super::GrpcHeaders;

pub mod success_rate {
    tonic::include_proto!("success_rate");
}
pub type DynamicRoutingResult<T> = CustomResult<T, DynamicRoutingError>;

/// Dynamic Routing Errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum DynamicRoutingError {
    /// The required input is missing
    #[error("Missing Required Field : {field} for building the Dynamic Routing Request")]
    MissingRequiredField {
        /// The required field name
        field: String,
    },
    /// Error from Dynamic Routing Server while performing success_rate analysis
    #[error("Error from Dynamic Routing Server while perfrming success_rate analysis : {0}")]
    SuccessRateBasedRoutingFailure(String),

    /// Error from Dynamic Routing Server while performing contract based routing
    #[error("Error from Dynamic Routing Server while performing contract based routing: {0}")]
    ContractBasedRoutingFailure(String),
    /// Error from Dynamic Routing Server while perfrming elimination
    #[error("Error from Dynamic Routing Server while perfrming elimination : {0}")]
    EliminationRateRoutingFailure(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CurrentBlockThreshold {
    pub duration_in_mins: Option<u64>,
    pub max_total_count: Option<u64>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SuccessRateSpecificityLevel {
    #[default]
    Merchant,
    Global,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SuccessBasedRoutingConfig {
    pub min_aggregates_size: Option<u32>,
    pub default_success_rate: Option<f64>,
    pub max_aggregates_size: Option<u32>,
    pub current_block_threshold: Option<CurrentBlockThreshold>,
    #[serde(default)]
    pub specificity_level: SuccessRateSpecificityLevel,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum DynamicRoutingConfigParams {
    PaymentMethod,
    PaymentMethodType,
    AuthenticationType,
    Currency,
    Country,
    CardNetwork,
    CardBin,
}


#[async_trait::async_trait]
pub trait SuccessBasedDynamicRouting: dyn_clone::DynClone + Send + Sync {
    async fn calculate_success_rate(
        &self,
        id: String,
        success_rate_based_config: SuccessBasedRoutingConfig,
        params: String,
        connector_list: Vec<Key>,
        headers: GrpcHeaders,
    ) -> DynamicRoutingResult<CalSuccessRateResponse>;

    async fn update_success_rate(
        &self,
        id: String,
        success_rate_based_config: SuccessBasedRoutingConfig,
        params: String,
        connector_list: Vec<LabelWithStatus>,
        headers: GrpcHeaders,
    ) -> DynamicRoutingResult<UpdateSuccessRateWindowResponse>;

    async fn invalidate_redis_keys(
        &self,
        id: String,
        headers: GrpcHeaders,
    ) -> DynamicRoutingResult<InvalidateWindowsResponse>;
}

#[async_trait::async_trait]
impl SuccessBasedDynamicRouting for SuccessRateCalculatorClient<Client> {
    async fn calculate_success_rate(
        &self,
        id: String,
        success_rate_based_config: SuccessBasedRoutingConfig,
        params: String,
        connector_list: Vec<Key>,
        headers: GrpcHeaders
    ) -> DynamicRoutingResult<CalSuccessRateResponse> {
        let connector_list = connector_list.into_iter().map(|key| key.0).collect();
    
        let config = foreign_try_from(success_rate_based_config).map_err(|err| {
            DynamicRoutingError::SuccessRateBasedRoutingFailure(err.to_string())
        })?;
    
        let request = create_grpc_request(
            CalSuccessRateRequest {
                id,
                params,
                labels: connector_list,
                config: Some(config),
            },
            headers,
        );
    
        let response = self
            .clone()
            .fetch_success_rate(request)
            .await
            .change_context(DynamicRoutingError::SuccessRateBasedRoutingFailure(
                "Error while fetching success rate".to_string(),
            ))?
            .into_inner();
    
        Ok(response)
    }

    async fn update_success_rate(
        &self,
        id: String,
        success_rate_based_config: SuccessBasedRoutingConfig,
        params: String,
        connector_list_with_status: Vec<LabelWithStatus>,
        headers: GrpcHeaders
    ) -> DynamicRoutingResult<UpdateSuccessRateWindowResponse> {

        let config = foreign_try_from_2(success_rate_based_config).map_err(|err| {
            DynamicRoutingError::SuccessRateBasedRoutingFailure(err.to_string())
        })?;

        // let connector_list = connector_list_with_status.into_iter().map(|key| key.0).collect();

        let request = create_grpc_request(
            UpdateSuccessRateWindowRequest {
                id,
                params,
                labels_with_status: connector_list_with_status.clone(),
                config: Some(config),
                global_labels_with_status: connector_list_with_status.clone(),
            },
            headers,
        );

        let response = self
            .clone()
            .update_success_rate_window(request)
            .await
            .change_context(DynamicRoutingError::SuccessRateBasedRoutingFailure(
                "Error while updating success rate".to_string(),
            ))?
            .into_inner();

        Ok(response)
    }

    async fn invalidate_redis_keys(
        &self,
        id: String,
        headers: GrpcHeaders,
    ) -> DynamicRoutingResult<InvalidateWindowsResponse> {
        let request = create_grpc_request(InvalidateWindowsRequest { id }, headers);

        let response = self
            .clone()
            .invalidate_windows(request)
            .await
            .change_context(DynamicRoutingError::SuccessRateBasedRoutingFailure(
                "Error while invalidating redis keys".to_string(),
            ))?
            .into_inner();

        Ok(response)
    }
}

fn get_required_value<T>(
    value: Option<T>,
    field_name: &'static str,
) -> CustomResult<T, DynamicRoutingError> 
    where T: std::fmt::Debug {
    match value {
        Some(v) => Ok(v),
        None => Err(DynamicRoutingError::MissingRequiredField {
            field: field_name.to_string(),
        })
        .attach_printable(format!("Missing required field {field_name} in {value:?}")),
    }
}

type Error = error_stack::Report<DynamicRoutingError>;

fn foreign_try_from(config: SuccessBasedRoutingConfig) -> Result<CalSuccessRateConfig, Error> {
    Ok(CalSuccessRateConfig {
        min_aggregates_size: get_required_value(config.min_aggregates_size, "min_aggregates_size")
            .change_context(DynamicRoutingError::MissingRequiredField {
                field: "min_aggregates_size".to_string(),
            })?,
        default_success_rate: get_required_value(config.default_success_rate, "default_success_rate")
            .change_context(DynamicRoutingError::MissingRequiredField{
                field: "default_success_rate".to_string(),
            })?,
        specificity_level: match config.specificity_level {
            SuccessRateSpecificityLevel::Merchant => Some(ProtoSpecificityLevel::Entity.into()),
            SuccessRateSpecificityLevel::Global => Some(ProtoSpecificityLevel::Global.into())
        },
    })
}

// fn foreign_try_from_2(config: SuccessBasedRoutingConfig) -> Result<UpdateSuccessRateWindowConfig, Error> {
//     Ok(UpdateSuccessRateWindowConfig {
//         max_aggregates_size: get_required_value(config.max_aggregates_size, "max_aggregates_size")
//             .change_context(DynamicRoutingError::MissingRequiredField {
//                 field: "max_aggregates_size".to_string(),
//             })?,
//         current_block_threshold: Some(get_required_value(config.current_block_threshold, "current_block_threshold")
//             .change_context(DynamicRoutingError::MissingRequiredField {
//                 field: "current_block_threshold".to_string(),
//             })?)
//     })
// }

fn foreign_try_from_2(config: SuccessBasedRoutingConfig) -> Result<UpdateSuccessRateWindowConfig, Error> {
    Ok(UpdateSuccessRateWindowConfig {
        max_aggregates_size: get_required_value(config.max_aggregates_size, "max_aggregates_size")
            .change_context(DynamicRoutingError::MissingRequiredField {
                field: "max_aggregates_size".to_string(),
            })?,
        current_block_threshold: config.current_block_threshold
            .map(|current_block_threshold| {
                success_rate::CurrentBlockThreshold {
                    duration_in_mins: current_block_threshold.duration_in_mins,
                    max_total_count: get_required_value(current_block_threshold.max_total_count, "max_total_count")
                        .change_context(DynamicRoutingError::MissingRequiredField {
                            field: "max_total_count".to_string(),
                        }).unwrap(),
                }
            })
    })
}

// fn foreign_try_from_3(current_threshold: CurrentBlockThreshold) -> Result<CurrentBlockThreshold, Error> {
//     Ok(CurrentBlockThreshold {
//         duration_in_mins: current_threshold.duration_in_mins,
//         max_total_count: current_threshold
//             .max_total_count
//             .get_required_value(current_threshold.max_total_count, "max_total_count")
//             .change_context(DynamicRoutingError::MissingRequiredField {
//                 field: "max_total_count".to_string(),
//             })?,
//     })
// }