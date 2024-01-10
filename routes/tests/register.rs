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
	register::{register_para, RegistrationData},
	Error,
};
use shared::{
	payment::PaymentError,
	registry::{registered_para, registered_paras},
};
use types::RelayChain::*;

mod mock;
use mock::{mock_para, MockEnvironment};

const PARA_2001_PAYMENT: BlockNumber = 8625079;

#[test]
fn register_works() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let mut para = mock_para(Polkadot, 2001);
		let registration_data =
			RegistrationData { para: para.clone(), payment_block_number: Some(PARA_2001_PAYMENT) };

		let response = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&registration_data).unwrap())
			.dispatch();

		assert_eq!(response.status(), Status::Ok);

		let response_para = registered_para(Polkadot, 2001).unwrap();

		// Set the `last_payment_timestamp` to the proper value.
		para.last_payment_timestamp = response_para.last_payment_timestamp;

		// Ensure the parachain is properly registered:
		assert_eq!(registered_paras(), vec![para.clone()]);
		assert_eq!(response_para, para);
	});
}

#[test]
fn cannot_register_same_para_twice() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = mock_para(Polkadot, 2001);
		let registration_data =
			RegistrationData { para, payment_block_number: Some(PARA_2001_PAYMENT) };

		let register = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&registration_data).unwrap());

		// Cannot register the same para twice:
		assert_eq!(register.clone().dispatch().status(), Status::Ok);
		assert_eq!(parse_err_response(register.dispatch()), Error::AlreadyRegistered);
	});
}

#[test]
fn providing_no_payment_info_fails() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = mock_para(Polkadot, 2020);
		let registration_data = RegistrationData { para, payment_block_number: None };

		let response = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&registration_data).unwrap())
			.dispatch();

		assert_eq!(parse_err_response(response), Error::PaymentRequired);
	});
}

#[test]
fn providing_non_finalized_payment_block_number_fails() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = mock_para(Polkadot, 2020);
		let registration_data = RegistrationData { para, payment_block_number: Some(99999999) };

		let response = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&registration_data).unwrap())
			.dispatch();

		assert_eq!(
			parse_err_response(response),
			Error::PaymentValidationError(PaymentError::Unfinalized)
		);
	});
}

#[test]
fn payment_not_found_works() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		// We are registering para 2020, but the payment is for para 2001.
		let para = mock_para(Polkadot, 2020);
		let registration_data =
			RegistrationData { para, payment_block_number: Some(PARA_2001_PAYMENT) };

		let response = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&registration_data).unwrap())
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
