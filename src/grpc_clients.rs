pub mod health_check_client;
pub mod success_rate_client;
use std::fmt::Debug;
pub const TENANT_HEADER: &str = "x-tenant-id";
pub const X_REQUEST_ID: &str = "x-request-id";
#[derive(Debug)]
pub struct GrpcHeaders {
    /// Tenant id
    pub tenant_id: String,
    /// Request id
    pub request_id: Option<String>,
}
pub trait AddHeaders {
    fn add_headers_to_grpc_request(&mut self, headers: GrpcHeaders);
}
impl<T> AddHeaders for tonic::Request<T> {
    #[track_caller]
    fn add_headers_to_grpc_request(&mut self, headers: GrpcHeaders) {
        headers.tenant_id
            .parse()
            .map(|tenant_id| {
                self
                    .metadata_mut()
                    .append(TENANT_HEADER, tenant_id)
            }
            )
            .ok();

        headers.request_id.map(|request_id| {
            request_id
                .parse()
                .map(|request_id| {
                    self
                        .metadata_mut()
                        .append(X_REQUEST_ID, request_id)
                })
                
                .ok();
        });
    }
}
pub fn create_grpc_request<T: Debug>(message: T, headers: GrpcHeaders) -> tonic::Request<T> {
    let mut request = tonic::Request::new(message);
    request.add_headers_to_grpc_request(headers);

    request
}