// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::{str, fs, fmt};
use std::time::Duration;
use util::{Address, U256, version_data};
use util::journaldb::Algorithm;
use ethcore::spec::Spec;
use ethcore::ethereum;
use ethcore::client::Mode;
use ethcore::miner::{GasPricer, GasPriceCalibratorOptions};
use user_defaults::UserDefaults;

#[derive(Debug, PartialEq)]
pub enum SpecType {
	Foundation,
	Morden,
	Ropsten,
	Kovan,
	Olympic,
	Classic,
	Expanse,
	Dev,
	Custom(String),
}

impl Default for SpecType {
	fn default() -> Self {
		SpecType::Foundation
	}
}

impl str::FromStr for SpecType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let spec = match s {
			"foundation" | "frontier" | "homestead" | "mainnet" => SpecType::Foundation,
			"frontier-dogmatic" | "homestead-dogmatic" | "classic" => SpecType::Classic,
			"morden" | "classic-testnet" => SpecType::Morden,
			"ropsten" => SpecType::Ropsten,
			"kovan" | "testnet" => SpecType::Kovan,
			"olympic" => SpecType::Olympic,
			"expanse" => SpecType::Expanse,
			"dev" => SpecType::Dev,
			other => SpecType::Custom(other.into()),
		};
		Ok(spec)
	}
}

impl fmt::Display for SpecType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match *self {
			SpecType::Foundation => "foundation",
			SpecType::Morden => "morden",
			SpecType::Ropsten => "ropsten",
			SpecType::Olympic => "olympic",
			SpecType::Classic => "classic",
			SpecType::Expanse => "expanse",
			SpecType::Kovan => "kovan",
			SpecType::Dev => "dev",
			SpecType::Custom(ref custom) => custom,
		})
	}
}

impl SpecType {
	pub fn spec(&self) -> Result<Spec, String> {
		match *self {
			SpecType::Foundation => Ok(ethereum::new_foundation()),
			SpecType::Morden => Ok(ethereum::new_morden()),
			SpecType::Ropsten => Ok(ethereum::new_ropsten()),
			SpecType::Olympic => Ok(ethereum::new_olympic()),
			SpecType::Classic => Ok(ethereum::new_classic()),
			SpecType::Expanse => Ok(ethereum::new_expanse()),
			SpecType::Kovan => Ok(ethereum::new_kovan()),
			SpecType::Dev => Ok(Spec::new_instant()),
			SpecType::Custom(ref filename) => {
				let file = fs::File::open(filename).map_err(|_| "Could not load specification file.")?;
				Spec::load(file)
			}
		}
	}

	pub fn legacy_fork_name(&self) -> Option<String> {
		match *self {
			SpecType::Classic => Some("classic".to_owned()),
			SpecType::Expanse => Some("expanse".to_owned()),
			_ => None,
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Pruning {
	Specific(Algorithm),
	Auto,
}

impl Default for Pruning {
	fn default() -> Self {
		Pruning::Auto
	}
}

impl str::FromStr for Pruning {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"auto" => Ok(Pruning::Auto),
			other => other.parse().map(Pruning::Specific),
		}
	}
}

impl Pruning {
	pub fn to_algorithm(&self, user_defaults: &UserDefaults) -> Algorithm {
		match *self {
			Pruning::Specific(algo) => algo,
			Pruning::Auto => user_defaults.pruning,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct ResealPolicy {
	pub own: bool,
	pub external: bool,
}

impl Default for ResealPolicy {
	fn default() -> Self {
		ResealPolicy {
			own: true,
			external: true,
		}
	}
}

impl str::FromStr for ResealPolicy {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (own, external) = match s {
			"none" => (false, false),
			"own" => (true, false),
			"ext" => (false, true),
			"all" => (true, true),
			x => return Err(format!("Invalid reseal value: {}", x)),
		};

		let reseal = ResealPolicy {
			own: own,
			external: external,
		};

		Ok(reseal)
	}
}

#[derive(Debug, PartialEq)]
pub struct AccountsConfig {
	pub iterations: u32,
	pub testnet: bool,
	pub password_files: Vec<String>,
	pub unlocked_accounts: Vec<Address>,
	pub enable_hardware_wallets: bool,
}

impl Default for AccountsConfig {
	fn default() -> Self {
		AccountsConfig {
			iterations: 10240,
			testnet: false,
			password_files: Vec::new(),
			unlocked_accounts: Vec::new(),
			enable_hardware_wallets: true,
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum GasPricerConfig {
	Fixed(U256),
	Calibrated {
		initial_minimum: U256,
		usd_per_tx: f32,
		recalibration_period: Duration,
	}
}

impl GasPricerConfig {
	pub fn initial_min(&self) -> U256 {
		match *self {
			GasPricerConfig::Fixed(ref min) => min.clone(),
			GasPricerConfig::Calibrated { ref initial_minimum, .. } => initial_minimum.clone(),
		}
	}
}

impl Default for GasPricerConfig {
	fn default() -> Self {
		GasPricerConfig::Calibrated {
			initial_minimum: 11904761856u64.into(),
			usd_per_tx: 0.0025f32,
			recalibration_period: Duration::from_secs(3600),
		}
	}
}

impl Into<GasPricer> for GasPricerConfig {
	fn into(self) -> GasPricer {
		match self {
			GasPricerConfig::Fixed(u) => GasPricer::Fixed(u),
			GasPricerConfig::Calibrated { usd_per_tx, recalibration_period, .. } => {
				GasPricer::new_calibrated(GasPriceCalibratorOptions {
					usd_per_tx: usd_per_tx,
					recalibration_period: recalibration_period,
				})
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct MinerExtras {
	pub author: Address,
	pub extra_data: Vec<u8>,
	pub gas_floor_target: U256,
	pub gas_ceil_target: U256,
	pub transactions_limit: usize,
	pub engine_signer: Address,
}

impl Default for MinerExtras {
	fn default() -> Self {
		MinerExtras {
			author: Default::default(),
			extra_data: version_data(),
			gas_floor_target: U256::from(4_700_000),
			gas_ceil_target: U256::from(6_283_184),
			transactions_limit: 1024,
			engine_signer: Default::default(),
		}
	}
}

/// 3-value enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Switch {
	/// True.
	On,
	/// False.
	Off,
	/// Auto.
	Auto,
}

impl Default for Switch {
	fn default() -> Self {
		Switch::Auto
	}
}

impl str::FromStr for Switch {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"on" => Ok(Switch::On),
			"off" => Ok(Switch::Off),
			"auto" => Ok(Switch::Auto),
			other => Err(format!("Invalid switch value: {}", other))
		}
	}
}

pub fn tracing_switch_to_bool(switch: Switch, user_defaults: &UserDefaults) -> Result<bool, String> {
	match (user_defaults.is_first_launch, switch, user_defaults.tracing) {
		(false, Switch::On, false) => Err("TraceDB resync required".into()),
		(_, Switch::On, _) => Ok(true),
		(_, Switch::Off, _) => Ok(false),
		(_, Switch::Auto, def) => Ok(def),
	}
}

pub fn fatdb_switch_to_bool(switch: Switch, user_defaults: &UserDefaults, _algorithm: Algorithm) -> Result<bool, String> {
	let result = match (user_defaults.is_first_launch, switch, user_defaults.fat_db) {
		(false, Switch::On, false) => Err("FatDB resync required".into()),
		(_, Switch::On, _) => Ok(true),
		(_, Switch::Off, _) => Ok(false),
		(_, Switch::Auto, def) => Ok(def),
	};
	result
}

pub fn mode_switch_to_bool(switch: Option<Mode>, user_defaults: &UserDefaults) -> Result<Mode, String> {
	Ok(switch.unwrap_or(user_defaults.mode.clone()))
}

#[cfg(test)]
mod tests {
	use util::journaldb::Algorithm;
	use user_defaults::UserDefaults;
	use super::{SpecType, Pruning, ResealPolicy, Switch, tracing_switch_to_bool};

	#[test]
	fn test_spec_type_parsing() {
		assert_eq!(SpecType::Foundation, "frontier".parse().unwrap());
		assert_eq!(SpecType::Foundation, "homestead".parse().unwrap());
		assert_eq!(SpecType::Foundation, "mainnet".parse().unwrap());
		assert_eq!(SpecType::Foundation, "foundation".parse().unwrap());
		assert_eq!(SpecType::Kovan, "testnet".parse().unwrap());
		assert_eq!(SpecType::Kovan, "kovan".parse().unwrap());
		assert_eq!(SpecType::Morden, "morden".parse().unwrap());
		assert_eq!(SpecType::Ropsten, "ropsten".parse().unwrap());
		assert_eq!(SpecType::Olympic, "olympic".parse().unwrap());
		assert_eq!(SpecType::Classic, "classic".parse().unwrap());
		assert_eq!(SpecType::Morden, "classic-testnet".parse().unwrap());
	}

	#[test]
	fn test_spec_type_default() {
		assert_eq!(SpecType::Foundation, SpecType::default());
	}

	#[test]
	fn test_spec_type_display() {
		assert_eq!(format!("{}", SpecType::Foundation), "foundation");
		assert_eq!(format!("{}", SpecType::Ropsten), "ropsten");
		assert_eq!(format!("{}", SpecType::Morden), "morden");
		assert_eq!(format!("{}", SpecType::Olympic), "olympic");
		assert_eq!(format!("{}", SpecType::Classic), "classic");
		assert_eq!(format!("{}", SpecType::Expanse), "expanse");
		assert_eq!(format!("{}", SpecType::Kovan), "kovan");
		assert_eq!(format!("{}", SpecType::Dev), "dev");
		assert_eq!(format!("{}", SpecType::Custom("foo/bar".into())), "foo/bar");
	}

	#[test]
	fn test_pruning_parsing() {
		assert_eq!(Pruning::Auto, "auto".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::Archive), "archive".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::EarlyMerge), "light".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::OverlayRecent), "fast".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::RefCounted), "basic".parse().unwrap());
	}

	#[test]
	fn test_pruning_default() {
		assert_eq!(Pruning::Auto, Pruning::default());
	}

	#[test]
	fn test_reseal_policy_parsing() {
		let none = ResealPolicy { own: false, external: false };
		let own = ResealPolicy { own: true, external: false };
		let ext = ResealPolicy { own: false, external: true };
		let all = ResealPolicy { own: true, external: true };
		assert_eq!(none, "none".parse().unwrap());
		assert_eq!(own, "own".parse().unwrap());
		assert_eq!(ext, "ext".parse().unwrap());
		assert_eq!(all, "all".parse().unwrap());
	}

	#[test]
	fn test_reseal_policy_default() {
		let all = ResealPolicy { own: true, external: true };
		assert_eq!(all, ResealPolicy::default());
	}

	#[test]
	fn test_switch_parsing() {
		assert_eq!(Switch::On, "on".parse().unwrap());
		assert_eq!(Switch::Off, "off".parse().unwrap());
		assert_eq!(Switch::Auto, "auto".parse().unwrap());
	}

	#[test]
	fn test_switch_default() {
		assert_eq!(Switch::default(), Switch::Auto);
	}

	fn user_defaults_with_tracing(first_launch: bool, tracing: bool) -> UserDefaults {
		let mut ud = UserDefaults::default();
		ud.is_first_launch = first_launch;
		ud.tracing = tracing;
		ud
	}

	#[test]
	fn test_switch_to_bool() {
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(true, true)).unwrap());
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(true, false)).unwrap());
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(false, true)).unwrap());
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(false, false)).unwrap());

		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(true, true)).unwrap());
		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(true, false)).unwrap());
		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(false, true)).unwrap());
		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(false, false)).is_err());
	}
}
