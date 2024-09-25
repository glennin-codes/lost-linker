use std::cell::RefCell;

use candid::{ CandidType, Decode, Deserialize, Encode, Principal };

use ic_stable_structures::{
    memory_manager::{ MemoryId, MemoryManager, VirtualMemory },
    BoundedStorable,
    Cell,
    DefaultMemoryImpl,
    StableBTreeMap,
    Storable,
};
use std::borrow::Cow;
use std::fmt;



type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;



#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum Status {
  Lost,
  Found,
}


#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum Reporter {
  LoserId(Principal), // The user who lost the item
  FounderId(Principal), // The user who found the item
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct  Search{
 pub category: Option<Category>,
 pub location: Option<String>,
 pub description: Option<String>
}
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum Category {
  Personal,
  Electronics,
  Documents,
  Clothing,
  Jewelry,
  Other, // Catch-all for categories not predefined
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Data {
    pub message: String,
    pub item: Item,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Item {
 pub id: u64,
 pub date: u64,
 pub description: String,
 pub category: Category,
 pub location: String,
 pub status: Status,
 pub reporter: Reporter,
 pub phone:u32
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Payload {
 pub description: String,
 pub category: Category,
 pub location: String,
 pub status: Status,
 pub phone:u32
}
#[derive(Debug, CandidType, Deserialize, Clone)]
pub enum CustomError {
  InvalidInput(String),
  StorageError(String),
  NotificationNotFound
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CustomError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            CustomError::NotificationNotFound => write!(f,"NO Notification found")
        }
    }
}

impl Storable for Item {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(
            Encode!(self).unwrap_or_else(|e| {
                ic_cdk::api::trap(&format!("Failed to encode Item: {:?}", e));
            })
        )
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap_or_else(|e| {
            ic_cdk::api::trap(&format!("Failed to decode Item: {:?}", e));
        })
    }
}

impl BoundedStorable for Item {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
   pub  static ITEM_ID: RefCell<IdCell> = RefCell::new(
        IdCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0
        ).expect("Cannot create a counter")
    );

   pub  static STORAGE: RefCell<StableBTreeMap<u64, Item, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );
}
