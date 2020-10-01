#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use sp_runtime::{
	traits::{CheckedConversion, MaybeSerializeDeserialize, SaturatedConversion},
	DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	prelude::*,
	result,
};

use xcm::v0::{Error, Junction, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{LocationConversion, MatchesFungible, TransactAsset};

use frame_support::debug;

pub trait CurrencyIdConversion<CurrencyId> {
	fn from_asset(asset: &MultiAsset) -> Option<CurrencyId>;
}

pub struct MultiCurrencyAdapter<MultiCurrency, Matcher, AccountIdConverter, AccountId, CurrencyIdConverter, CurrencyId>(
	PhantomData<MultiCurrency>,
	PhantomData<Matcher>,
	PhantomData<AccountIdConverter>,
	PhantomData<AccountId>,
	PhantomData<CurrencyIdConverter>,
	PhantomData<CurrencyId>,
);

impl<
		MultiCurrency: orml_traits::MultiCurrency<AccountId, CurrencyId = CurrencyId>,
		Matcher: MatchesFungible<MultiCurrency::Balance>,
		AccountIdConverter: LocationConversion<AccountId>,
		AccountId: sp_std::fmt::Debug,
		CurrencyIdConverter: CurrencyIdConversion<CurrencyId>,
		CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
	> TransactAsset
	for MultiCurrencyAdapter<MultiCurrency, Matcher, AccountIdConverter, AccountId, CurrencyIdConverter, CurrencyId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
		debug::info!("------------------------------------------------");
		debug::info!(">>> trying deposit. asset: {:?}, location: {:?}", asset, location);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency_id = CurrencyIdConverter::from_asset(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency_id);
		let amount = Matcher::matches_fungible(&asset).ok_or(())?.saturated_into();
		debug::info!("amount: {:?}", amount);
		let balance_amount = amount.try_into().map_err(|_| ())?;
		debug::info!("balance amount: {:?}", balance_amount);
		MultiCurrency::deposit(currency_id, &who, balance_amount).map_err(|_| ())?;
		debug::info!(">>> success deposit.");
		debug::info!("------------------------------------------------");
		Ok(())
	}

	fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> result::Result<MultiAsset, Error> {
		debug::info!("------------------------------------------------");
		debug::info!(">>> trying withdraw. asset: {:?}, location: {:?}", asset, location);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency_id = CurrencyIdConverter::from_asset(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency_id);
		let amount = Matcher::matches_fungible(&asset).ok_or(())?.saturated_into();
		debug::info!("amount: {:?}", amount);
		let balance_amount = amount.try_into().map_err(|_| ())?;
		debug::info!("balance amount: {:?}", balance_amount);
		MultiCurrency::withdraw(currency_id, &who, balance_amount).map_err(|_| ())?;
		debug::info!(">>> success withdraw.");
		debug::info!("------------------------------------------------");
		Ok(asset.clone())
	}
}

pub trait XcmHandler {
	type Origin;
	type Xcm;
	fn execute(origin: Self::Origin, xcm: Self::Xcm) -> DispatchResult;
}

pub struct IsConcreteWithGeneralKey<CurrencyId>(PhantomData<CurrencyId>);
impl<CurrencyId: TryFrom<Vec<u8>>, B: TryFrom<u128>> MatchesFungible<B> for IsConcreteWithGeneralKey<CurrencyId> {
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		if let MultiAsset::ConcreteFungible { id, amount } = a {
			if let Some(Junction::GeneralKey(key)) = id.last() {
				if TryInto::<CurrencyId>::try_into(key.clone()).is_ok() {
					return CheckedConversion::checked_from(*amount);
				}
			}
		}
		None
	}
}
