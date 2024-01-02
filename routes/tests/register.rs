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

// TODO: https://github.com/RegionX-Labs/CorespaceWeigher/issues/11

#[cfg(test)]
use rocket::{
	http::{ContentType, Status},
	local::blocking::{Client, LocalResponse},
	routes,
};
use routes::{register::register_para, Error};
use shared::registry::{registered_para, registered_paras};
use types::RelayChain::*;

mod mock;
use mock::{mock_para, MockEnvironment};

#[test]
fn register_works() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = mock_para(Polkadot, 2001);

		let response = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&para).unwrap())
			.dispatch();

		assert_eq!(response.status(), Status::Ok);

		// Ensure the parachain is properly registered:
		assert_eq!(registered_paras(), vec![para.clone()]);
		assert_eq!(registered_para(Polkadot, 2001), Some(para));
	});
}

#[test]
fn cannot_register_same_para_twice() {
	MockEnvironment::default().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![register_para]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = mock_para(Polkadot, 2001);

		let register = client
			.post("/register_para")
			.header(ContentType::JSON)
			.body(serde_json::to_string(&para).unwrap());

		// Cannot register the same para twice:
		assert_eq!(register.clone().dispatch().status(), Status::Ok);
		assert_eq!(parse_err_response(register.dispatch()), Error::AlreadyRegistered);
	});
}

fn parse_err_response<'a>(response: LocalResponse<'a>) -> Error {
	let body = response.into_string().unwrap();
	body.into()
}
