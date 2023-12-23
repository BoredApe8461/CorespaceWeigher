use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;
use rocket::Response;

/// Web API for interacting with the Consumption Tracker service.
///
/// This API exposes two main endpoints:
/// - `/consumption`: Used to query consumption data associated with a parachain.
/// - `/register`: Used to register a parachain for consumption tracking.

#[derive(Debug)]
pub enum Error {
    /// Cannot register an already registered parachain.
    AlreadyRegistered,
    /// Tried to get the consumption of a parachain that is not registered.
    NotRegistered,
    /// Indicates that the consumption data for the parachain was not found.
    ///
    /// This should be only encountered if the consumption file has not been generated yet,
    /// which is only possible if the parachain has been registered within the last few seconds.
    ConsumptionDataNotFound,
    /// The stored data is invalid. This should never really happen.
    InvalidData,
    /// Failed to find the parachains data. This isn't a user error, but a bug in the code itself.
    ParasDataNotFound,
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> Result<Response<'static>, Status> {
        let body = format!("Error: {:?}", self);
        Response::build()
            .status(Status::InternalServerError)
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}

pub mod consumption;
pub mod register;
