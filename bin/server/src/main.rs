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

/// Web API for interacting with the Consumption Tracker service.
///
/// This API exposes two main endpoints:
/// - `/consumption`: Used to query consumption data associated with a parachain.
/// - `/register`: Used to register a parachain for consumption tracking.
use rocket_cors::CorsOptions;
use routes::{
	consumption::consumption, extend_subscription::extend_subscription, register::register_para,
	registry::registry,
};

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
	rocket::build()
		.attach(CorsOptions::default().to_cors().unwrap())

		.mount("/", routes![consumption, register_para, registry, extend_subscription])

}
