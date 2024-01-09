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

use crate::Error;
use rocket::get;
use shared::registry::registered_paras;

/// Query all the registered parachains.
#[get("/registry")]
pub fn registry() -> Result<String, Error> {
	let registered_paras = registered_paras();

	serde_json::to_string(&registered_paras).map_err(|_| Error::InvalidData)
}
