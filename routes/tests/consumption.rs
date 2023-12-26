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

#[cfg(test)]
use rocket::{
	http::Status,
	local::blocking::{Client, LocalResponse},
	routes,
};
use routes::consumption::consumption;
use types::{RelayChain::*, WeightConsumption};

mod mock;
use mock::{mock_consumption, mock_para, MockEnvironment};

#[test]
fn getting_all_consumption_data_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![consumption]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = mock_para(Polkadot, 2000);
		let response = client.get("/consumption/polkadot/2000").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let consumption_data = parse_response(response);
		assert_eq!(consumption_data, mock_consumption().get(&para).unwrap().clone());
	});
}

pub fn parse_response<'a>(response: LocalResponse<'a>) -> Vec<WeightConsumption> {
	let body = response.into_string().unwrap();
	serde_json::from_str(&body).expect("can't parse value")
}
