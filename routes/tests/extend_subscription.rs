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

use polkadot_core_primitives::BlockNumber;
use rocket::{
	http::{ContentType, Status},
	local::blocking::{Client, LocalResponse},
	routes,
};
use routes::{
	extend_subscription::{extend_subscription, ExtendSubscriptionData},
	Error,
};
use shared::{chaindata::get_para, payment::PaymentError, registry::registered_para};
use types::RelayChain::*;

mod mock;
use mock::MockEnvironment;

const PARA_2000_PAYMENT: BlockNumber = 8624975;

#[test]
fn extend_subscription_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![extend_subscription]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2000).unwrap();
		let extend_subscription = ExtendSubscriptionData {
			para: (para.relay_chain.clone(), para.para_id),
			payment_block_number: PARA_2000_PAYMENT,
		};

		let response = client
			.post("/extend-subscription")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&extend_subscription).unwrap())
			.dispatch();

		assert_eq!(response.status(), Status::Ok);

		let registered = registered_para(Polkadot, 2000).unwrap();
		// Ensure the `expiry_timestamp` got updated:
		assert!(registered.expiry_timestamp != para.expiry_timestamp);
	});
}

#[test]
fn cannot_extend_subscription_for_unregistered() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![extend_subscription]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let extend_subscription = ExtendSubscriptionData {
			para: (Polkadot, 2006),
			payment_block_number: PARA_2000_PAYMENT,
		};

		let response = client
			.post("/extend-subscription")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&extend_subscription).unwrap())
			.dispatch();

		assert_eq!(parse_err_response(response), Error::NotRegistered);
	});
}

#[test]
fn providing_non_finalized_payment_block_number_fails() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![extend_subscription]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2000).unwrap();
		let extend_subscription = ExtendSubscriptionData {
			para: (para.relay_chain.clone(), para.para_id),
			payment_block_number: 99999999,
		};

		let response = client
			.post("/extend-subscription")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&extend_subscription).unwrap())
			.dispatch();

		assert_eq!(
			parse_err_response(response),
			Error::PaymentValidationError(PaymentError::Unfinalized)
		);
	});
}

#[test]
fn payment_not_found_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![extend_subscription]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2004).unwrap();
		// We are extending the subscription for para 2004, but the payment is for para 2000.
		let extend_subscription = ExtendSubscriptionData {
			para: (para.relay_chain.clone(), para.para_id),
			payment_block_number: PARA_2000_PAYMENT,
		};

		let response = client
			.post("/extend-subscription")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&extend_subscription).unwrap())
			.dispatch();

		assert_eq!(
			parse_err_response(response),
			Error::PaymentValidationError(PaymentError::NotFound)
		);
	});
}

fn parse_err_response<'a>(response: LocalResponse<'a>) -> Error {
	let body = response.into_string().unwrap();
	body.into()
}
