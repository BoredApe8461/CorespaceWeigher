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
use routes::{consumption::consumption, Error};
use shared::{chaindata::get_para, registry::update_registry, reset_mock_environment};
use types::{RelayChain::*, WeightConsumption};

mod mock;
use mock::{mock_consumption, MockEnvironment};

#[test]
fn getting_all_consumption_data_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![consumption]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2000).unwrap();
		let response = client.get("/consumption/polkadot/2000").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let consumption_data = parse_ok_response(response);
		assert_eq!(consumption_data, mock_consumption().get(&para).unwrap().clone());
	});
}

#[test]
fn parachain_not_found_handled() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![consumption]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let response = client.get("/consumption/polkadot/42").dispatch();
		assert_eq!(response.status(), Status::InternalServerError);

		let err = parse_err_response(response);
		assert_eq!(err, Error::NotRegistered);
	});
}

#[test]
fn consumption_data_not_found_handled() {
	// We run this test outside the mock environment which means the consumption data state won't
	// get inititalized.

	let rocket = rocket::build().mount("/", routes![consumption]);
	let client = Client::tracked(rocket).expect("valid rocket instance");

	// Register a parachain without storing any consumption data.
	assert!(update_registry(vec![get_para(Polkadot, 2000).unwrap()]).is_ok());

	let response = client.get("/consumption/polkadot/2000").dispatch();
	assert_eq!(response.status(), Status::InternalServerError);

	let err = parse_err_response(response);
	assert_eq!(err, Error::ConsumptionDataNotFound);

	reset_mock_environment();
}

#[test]
fn pagination_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![consumption]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2000).unwrap();
		let mock_data = mock_consumption().get(&para).unwrap().clone();

		// CASE 1: Limit response size by setting page size
		let response = client.get("/consumption/polkadot/2000?page_size=1").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let consumption_data = parse_ok_response(response);
		// Should only contain the first consumption data.
		assert_eq!(consumption_data, vec![mock_data.first().unwrap().clone()]);

		// CASE 2: Specifying the page without page size will still show all the data.
		let response = client.get("/consumption/polkadot/2000?page=0").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let consumption_data = parse_ok_response(response);
		// Should only contain the first consumption data.
		assert_eq!(consumption_data, mock_data);

		// CASE 3: Specifying the page and page size works.
		let response = client.get("/consumption/polkadot/2000?page=1&page_size=2").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let consumption_data = parse_ok_response(response);
		// Should skip the first page and take the second one.
		assert_eq!(
			consumption_data,
			mock_data.into_iter().skip(2).take(2).collect::<Vec<WeightConsumption>>()
		);

		// CASE 4: An out-of-bound page and page size will return an empty vector.
		let response = client.get("/consumption/polkadot/2000?page=69&page_size=42").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let consumption_data = parse_ok_response(response);
		assert!(consumption_data.is_empty());
	});
}

#[test]
fn timestamp_based_filtering_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![consumption]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2000).unwrap();
		let mock_data = mock_consumption().get(&para).unwrap().clone();

		// CASE 1: setting the starting timestamp filters out the data.
		let start_timestamp = 6;
		let response = client.get("/consumption/polkadot/2000?start=6").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let response_data = parse_ok_response(response);
		let expected_data = mock_data
			.clone()
			.into_iter()
			.filter(|c| c.timestamp >= start_timestamp)
			.collect::<Vec<WeightConsumption>>();

		// Should only contain the consumption where the timestamp is greater than or equal to 6.
		assert_eq!(response_data, expected_data);

		// CASE 2: setting the ending timestamp filters out the data.
		let end_timestamp = 12;
		let response = client.get("/consumption/polkadot/2000?end=12").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let response_data = parse_ok_response(response);
		let expected_data = mock_data
			.clone()
			.into_iter()
			.filter(|c| c.timestamp <= end_timestamp)
			.collect::<Vec<WeightConsumption>>();

		// Should only contain the consumption where the timestamp is less than or equal to 12.
		assert_eq!(response_data, expected_data);

		// CASE 3: setting the both start and ending timestamp filters out the data.
		let start_timestamp = 6;
		let end_timestamp = 6;
		let response = client.get("/consumption/polkadot/2000?start=6&end=6").dispatch();
		assert_eq!(response.status(), Status::Ok);

		let response_data = parse_ok_response(response);
		let expected_data = mock_data
			.into_iter()
			.filter(|c| c.timestamp >= start_timestamp && c.timestamp <= end_timestamp)
			.collect::<Vec<WeightConsumption>>();

		assert_eq!(response_data, expected_data);
		// Should only contain one consumption data since the `start` and `end` are set to the same
		// value.
		assert!(response_data.len() == 1);
	});
}

#[test]
fn pagination_and_timestamp_filtering_works() {
	MockEnvironment::new().execute_with(|| {
		let rocket = rocket::build().mount("/", routes![consumption]);
		let client = Client::tracked(rocket).expect("valid rocket instance");

		let para = get_para(Polkadot, 2000).unwrap();
		let mock_data = mock_consumption().get(&para).unwrap().clone();

		// Combined Case: Filter by timestamp and paginate
		let start_timestamp = 6;
		let page_size = 2;
		let page_number = 1;
		let response = client
			.get(format!(
				"/consumption/polkadot/2000?start={}&page={}&page_size={}",
				start_timestamp, page_number, page_size
			))
			.dispatch();
		assert_eq!(response.status(), Status::Ok);

		let response_data = parse_ok_response(response);
		let expected_data = mock_data
			.into_iter()
			.filter(|c| c.timestamp >= start_timestamp)
			.skip(page_size * page_number)
			.take(page_size)
			.collect::<Vec<WeightConsumption>>();

		// Check if the data is filtered by timestamp and then paginated
		assert_eq!(response_data, expected_data);
	});
}

fn parse_ok_response<'a>(response: LocalResponse<'a>) -> Vec<WeightConsumption> {
	let body = response.into_string().unwrap();
	serde_json::from_str(&body).expect("can't parse value")
}

fn parse_err_response<'a>(response: LocalResponse<'a>) -> Error {
	let body = response.into_string().unwrap();
	body.into()
}
