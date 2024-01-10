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

use rocket::{
	http::Status,
	local::blocking::{Client, LocalResponse},
	routes,
};
use routes::registry::registry;
use types::{Parachain, RelayChain::*};

mod mock;
use mock::{mock_para, MockEnvironment};

#[test]
fn getting_registry_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![registry]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let response = client.get("/registry").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let mut registry = parse_ok_response(response);
		registry.sort_by_key(|p| p.para_id);

		assert_eq!(registry, vec![mock_para(Polkadot, 2000), mock_para(Polkadot, 2005)]);
	});
}

fn parse_ok_response<'a>(response: LocalResponse<'a>) -> Vec<Parachain> {
	let body = response.into_string().unwrap();
	serde_json::from_str(&body).expect("can't parse value")
}
