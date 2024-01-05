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

use rocket::{http::Status, response::Responder, Request, Response};
use serde::{Deserialize, Serialize};

const LOG_TARGET: &str = "server";

/// Web API for interacting with the Consumption Tracker service.
///
/// This API exposes two main endpoints:
/// - `/consumption`: Used to query consumption data associated with a parachain.
/// - `/register`: Used to register a parachain for consumption tracking.

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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
	/// The caller tried to register a parachain without payment.
	PaymentRequired,
	/// Failed to validate they payment.
	PaymentValidationFailed,
	/// The receipt is not referencing a finalized block.
	UnfinalizedPayment,
	/// The payment was not found in the specified block.
	PaymentNotFound,
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
			"PaymentValidationFailed" => Self::PaymentValidationFailed,
			"PaymentNotFound" => Self::PaymentNotFound,
			_ => panic!("UnknownError"),
		}
	}
}

pub mod consumption;
pub mod register;
