
use std::cell::RefCell;
use candid::{ CandidType, Decode, Deserialize, Encode, Principal };

use ic_cdk::api::time;
use ic_stable_structures::{
    memory_manager::{ MemoryId, MemoryManager, VirtualMemory },
    BoundedStorable,
    Cell,
    DefaultMemoryImpl,
    StableBTreeMap,
    Storable,
};
use std::borrow::Cow;

use crate::item::{CustomError, Item, Reporter};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Notification{
    id: u64,
    recipient: Principal,
    message: String,
    item_id: u64,
    created_at: u64,
    read: bool,
    phone:u32
}
impl Storable for Notification {
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
impl BoundedStorable for Notification {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
   pub  static  NOTIFICATION_ID: RefCell<IdCell> = RefCell::new(
        IdCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            2
        ).expect("Cannot create a counter")
    );

   pub  static NOTIFICATIONS: RefCell<StableBTreeMap<u64, Notification, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );
}


fn create_notification(recipient: Principal, message: String, item_id: u64,phone:u32)->(){
    let notification_id=NOTIFICATION_ID.with(|count|
    {
           let id = *count.borrow().get();
           let _ = count.borrow_mut().set(id + 1);
           id
    });

    let new_notification= Notification{
        id: notification_id,
        recipient,
        message,
        item_id,
        created_at: time(),
        read: false,
        phone
    };

    NOTIFICATIONS.with(|n|{
        n.borrow_mut().insert(notification_id, new_notification)
    });

}
pub fn notify_reporter(matched_item:&Item,new_item:&Item){
    match(&matched_item.reporter,&new_item.reporter){
        (Reporter::LoserId(loser),Reporter::FounderId(founder))=>  {
            let message = format!("Your lost item (ID: {}) has been found. Contact {:?} the finder for more information.", matched_item.id,matched_item.phone);
            create_notification(*loser, message, matched_item.id,matched_item.phone);
            let message = format!("The owner of the item you found (ID: {}) has been identified. Contact the owner {:?}", new_item.id,new_item.phone);
            create_notification(*founder, message, new_item.id,new_item.phone);
        },
        (Reporter::FounderId(founder) ,Reporter::LoserId(loser)) =>{
            let message = format!("The owner of the item you found (ID: {}) has been identified. Contact the owner {:?}", matched_item.id,matched_item.phone);
            create_notification(*founder, message, matched_item.id,matched_item.phone);
            
            let message = format!("Your lost item (ID: {}) has been found. Contact {} the finder for more information.", new_item.id,new_item.phone);
            create_notification(*loser, message, new_item.id,new_item.phone);
        },
        _ => {
            ic_cdk::println!("Unexpected reporter combination");
        }
    }
}
pub fn user_notifications(phone: u32) -> Vec<Notification> {
    NOTIFICATIONS.with(|notifications| {
        notifications
            .borrow()
            .iter()
            .filter(|(_, notification)| notification.phone == phone)
            .map(|(_, notification)| notification)
            .collect()
    })
}
// pub fn mark_notification_as_read(notification_id: u64) -> Result<String, CustomError> {
//     NOTIFICATIONS.with(|notifications| {
//         let mut notifications = notifications.borrow_mut();
//         if let Some(mut notification) = notifications.get(&notification_id) {
//             notification.read = true;
//             notifications.insert(notification_id, notification);
//             Ok("notification is read".to_string())
//         } else {
//             Err(CustomError::NotificationNotFound)
//         }
//     })
// }

pub fn delete_notification_function(notification_id: u64)->Result<String,CustomError>{
    NOTIFICATIONS.with(|not|{
        let mut  notifications=not.borrow_mut();
if let Some(notify)= notifications.get(&notification_id){
    notifications.remove(&notification_id);
    Ok(format!("notification (ID :{:?}) deleted succesfuly ",notify.id))
}else{
    Err(CustomError::NotificationNotFound)
}

})
    
}