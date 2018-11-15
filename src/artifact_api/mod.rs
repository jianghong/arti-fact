use std::collections::HashMap;

extern crate reqwest;

const CARD_SET_REQUEST_URL: &str =  "https://playartifact.com/cardset/";

pub const BASE_SET_ID: &str = "00";
pub const CALL_TO_ARMS_SET_ID: &str = "01";
pub const SET_IDS: &'static [&'static str] = &[BASE_SET_ID, CALL_TO_ARMS_SET_ID];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardSetRequest {
	cdn_root: String,
	url: String,
	expire_time: i32,
}

impl CardSetRequest {
	pub fn url(self) -> reqwest::Url {
		reqwest::Url::parse(&self.cdn_root).unwrap().join(&self.url).unwrap()
	}
}

impl PartialEq for CardSetRequest {
    fn eq(&self, other: &CardSetRequest) -> bool {
        self.cdn_root == other.cdn_root &&
        self.url == other.url &&
        self.expire_time == other.expire_time
    }	
}

#[derive(Debug)]
pub enum CardSetRequestError {
	InvalidSetID { kind: reqwest::UrlError },
	ReqwestError { kind: reqwest::Error },
}

impl CardSetRequestError {
	#[allow(dead_code)]
	fn to_string(self) -> String {
		match self {
			CardSetRequestError::InvalidSetID{kind} => kind.to_string(),
			CardSetRequestError::ReqwestError{kind} => kind.to_string(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslationSet {
	english: Option<String>,
}

impl TranslationSet {
	fn english_val(self) -> String {
		if let Some(english) = self.english {
			english
		} else {
			"".to_string()
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageSet {
	pub default: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetInfo {
	set_id: i32,
	pack_item_def: i32,
	name: TranslationSet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardReference {
	card_id: i32,
	ref_type: String,
	count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Rarity {
	Common,
	Uncommon,
	Rare,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
	card_id: i32,
	base_card_id: i32,
	card_type: String,
	card_name: TranslationSet,
	card_text: TranslationSet,
	pub mini_image: ImageSet,
	pub large_image: ImageSet,
	pub ingame_image: ImageSet,
	references: Vec<CardReference>,
	attack: Option<i32>,
	hit_points: Option<i32>,
	illustrator: Option<String>,
	gold_cost: Option<i32>,
	mana_cost: Option<i32>,
	sub_type: Option<String>,
	is_green: Option<bool>,
	is_red: Option<bool>,
	is_black: Option<bool>,
	is_blue: Option<bool>,
	item_def: Option<i32>,
	rarity: Option<Rarity>,
}

impl Card {
	pub fn print_item_info(self) {
		if let Some(gold) = self.gold_cost {
			println!("Name: {} / Gold: {}", self.card_name.english_val(), gold);
		}
	}
}

type CardList = Vec<Card>;

pub struct FindItemsParams {
	pub gold_cost: i32,
	pub include_hold: bool,
}

pub trait CardListFilterable {
	fn find_items(self, &FindItemsParams) -> CardList;
}

impl CardListFilterable for CardList {
	fn find_items(self, params: &FindItemsParams) -> CardList {
		self.into_iter()
			.filter(|card| {
				if let Some(card_gold_cost) = card.gold_cost {
					card.card_type == "Item" &&
					card_gold_cost == params.gold_cost ||
					(params.include_hold && card_gold_cost == params.gold_cost -1)
				} else {
					return false
				}
			})
			.collect::<CardList>()
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardSet {
	version: i32,
	set_info: SetInfo,
	pub card_list: CardList,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardSetResponse {
	pub card_set: CardSet,
}

#[derive(Debug)]
pub struct CardSetApi {
	cached_sets: HashMap<String, CardSetResponse>,
	client: reqwest::Client,
}

impl CardSetApi {
	pub fn new() -> Self {
		CardSetApi {
			cached_sets: HashMap::new(),
			client: reqwest::Client::new(),
		}
	}

	pub fn get_set(&mut self, set_id: &str) -> Result<CardSetResponse, CardSetRequestError> {
		if let Some(cached_set) = self.cached_sets.get(set_id.into()) {
			println!("Found cached set response for set {}", cached_set.card_set.set_info.set_id);
			return Ok(cached_set.clone());
		}

		println!("Fetching set_id {} from server...", set_id);
		self.get_set_request(set_id)
			.and_then(|card_set_request| reqwest::get(card_set_request.url()).map_err(|e| CardSetRequestError::ReqwestError{kind: e}))
			.and_then(|mut response| response.json().map_err(|e| CardSetRequestError::ReqwestError{kind: e}))
			.and_then(|card_set_response: CardSetResponse| {
				self.cached_sets.insert(
					set_id.into(),
					card_set_response.clone()
				);
				Ok(card_set_response)
			})
	}

	pub fn get_cards(&mut self) -> Result<CardList, CardSetRequestError> {
		let mut card_list: CardList = vec![];

		for set_id in SET_IDS {
			match self.get_set(set_id) {
				Ok(card_set_response) => card_list.append(&mut card_set_response.card_set.card_list.clone()),
				Err(e) => return Err(e)
			}
		}
		Ok(card_list)
	}

	fn get_set_request(&mut self, set_id: &str) -> Result<CardSetRequest, CardSetRequestError> {
		self.parse_url(set_id)
			.and_then(|url| reqwest::get(url).map_err(|e| CardSetRequestError::ReqwestError{kind: e}))
			.and_then(|mut response| response.json().map_err(|e| CardSetRequestError::ReqwestError{kind: e}))
	}

	fn parse_url(&mut self, set_id: &str) -> Result<reqwest::Url, CardSetRequestError> {
		reqwest::Url::parse(CARD_SET_REQUEST_URL).map_err(|e| CardSetRequestError::InvalidSetID{kind: e})
			.and_then(|base_url| base_url.join(set_id).map_err(|e| CardSetRequestError::InvalidSetID{kind: e}))
	}
}
