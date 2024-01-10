// This file is part of RegionX.
//
// RegionX is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// RegionX is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with RegionX.  If not, see <https://www.gnu.org/licenses/>.

//! Web API for interacting with the Consumption Tracker service.
//!
//! This API exposes the following endpoints:
//! - `/consumption`: Used to query consumption data associated with a parachain.
//! - `/register`: Used to register a parachain for consumption tracking.
//! - `/registry`: Used for querying all the registered parachains.

use rocket::{http::Status, response::Responder, Request, Response};
use serde::{Deserialize, Serialize};
use shared::payment::PaymentError;

const LOG_TARGET: &str = "server";

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Error {
	/// Cannot register an already registered parachain.
	AlreadyRegistered,
	/// The specified para is not registered.
	NotRegistered,
	/// Indicates that the consumption data for the parachain was not found.
	///
	/// This should be only encountered if the consumption file has not been generated yet,
	/// which is only possible if the parachain has been registered within the last few seconds.
	ConsumptionDataNotFound,
	/// The stored data is invalid. This should never really happen.
	InvalidData,
	/// The caller tried to register a parachain without payment.
	PaymentRequired,
	/// An error occured when trying to validate the payment.
	PaymentValidationError(PaymentError),
}

impl<'r> Responder<'r, 'static> for Error {
	fn respond_to(self, _: &'r Request<'_>) -> Result<Response<'static>, Status> {
		let body = format!("{:?}", self);
		Response::build()
			.status(Status::InternalServerError)
			.sized_body(body.len(), std::io::Cursor::new(body))
			.ok()
	}
}

impl From<String> for Error {
	fn from(v: String) -> Self {
		match v.as_str() {
			"AlreadyRegistered" => Self::AlreadyRegistered,
			"NotRegistered" => Self::NotRegistered,
			"ConsumptionDataNotFound" => Self::ConsumptionDataNotFound,
			"InvalidData" => Self::InvalidData,
			"PaymentRequired" => Self::PaymentRequired,
			// TODO: fix
			"PaymentValidationError(PaymentError::ValidationFailed)" =>
				Self::PaymentValidationError(PaymentError::ValidationFailed),
			"PaymentValidationError(PaymentError::Unfinalized)" =>
				Self::PaymentValidationError(PaymentError::Unfinalized),
			"PaymentValidationError(PaymentError::NotFound)" =>
				Self::PaymentValidationError(PaymentError::NotFound),
			_ => panic!("UnknownError"),
		}
	}
}

pub mod consumption;
//pub mod extend_subscription;
pub mod register;
pub mod registry;
