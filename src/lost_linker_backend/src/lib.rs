use item::{CustomError, Data, Item, Payload, Reporter, Search, Status, ITEM_ID, STORAGE};
use ic_cdk::{ api::time, caller, query, update };
use noification::{ delete_notification_function, notify_reporter, user_notifications, DeleteNoty, Notification, UserNoty};

mod item;
mod noification;

#[update]
fn report_item(Payload { description, category, location, status,phone }: Payload) -> Result<
    Data,
    CustomError
> {
    if description.is_empty() {
        return Err(CustomError::InvalidInput("Description cannot be empty".to_string()));
    }

    if location.is_empty() {
        return Err(CustomError::InvalidInput("Location cannot be empty".to_string()));
    }

    let item_id = ITEM_ID.with(|count| {
        let id = *count.borrow().get();
        let _ = count.borrow_mut().set(id + 1);
        id
    });
    let reporter = if status == Status::Lost {
        Reporter::LoserId(caller())
    } else {
        Reporter::FounderId(caller())
    };

    let new_item = Item {
        id: item_id,
        date: time(),
        description,
        category,
        location,
        status: status.clone(),
        reporter,
        phone
    };

    // Try to find a match based on the item status
    let item = match status {
        // Call match_items to check for a match
        Status::Lost => match_items(&new_item, status),
        Status::Found => match_items(&new_item, status),
    };

    // Handle the result of match_items
    match item {
        Some(i) => {
            // Match found
            notify_reporter(&i, &new_item);
            return Ok(Data { message: format!("Match found!"), item: i });
        }
        None => {
            // If no match is found, store the item and return a success Item
            STORAGE.with(|item| {
                item.borrow_mut().insert(new_item.id, new_item.clone());
            });
            // No match found
            Ok(Data {
                message: format!("No match found but  Item saved successfully"),
                item: new_item,
            })
        }
    }
}

fn match_items(reported_item: &Item, status: Status) -> Option<Item> {
    STORAGE.with(|storage| {
        let borrowed_items = storage.borrow();
        let items: Vec<Item> = borrowed_items
            .iter()
            .filter(|(_, item)| {
                match status {
                    Status::Lost =>
                        item.status == Status::Found &&
                            item.id != reported_item.id &&
                            item.category == reported_item.category,
                    Status::Found =>
                        item.status == Status::Lost &&
                            item.id != reported_item.id &&
                            item.category == reported_item.category,
                }
            })
            .map(|(_, item)| item)
            .collect();

        ic_cdk::println!(
            "the status matched to this item is {:?}\n.{:#?} potential matches",
            status,
            items
        );

        // Iterate through the filtered items to find a match
        for item in items {
            if
                item.location == reported_item.location &&
                description_match(&item.description, &reported_item.description)
            {
                ic_cdk::println!("Match found: {:?}", item);
                return Some(item);
            }
        }

        ic_cdk::println!("No matches found for the given item.");
        None
    })
}

// Helper function for description matching (basic implementation)
fn description_match(lost_desc: &str, found_desc: &str) -> bool {
    lost_desc.to_lowercase().contains(&found_desc.to_lowercase()) ||
        found_desc.to_lowercase().contains(&lost_desc.to_lowercase())
}

#[update]
fn search_lost_items(
   Search { category, location, description }: Search
) -> Result<Vec<Item>, CustomError> {
    
    STORAGE.with(|storage| {
        let borrowed_storage = storage.borrow();
        let mut results: Vec<Item> = borrowed_storage
            .iter()
            .filter(|(_, item)| {
                item.status==Status::Found &&
                category.as_ref().map_or(true, |c| item.category == *c) &&
                    location
                        .as_ref()
                        .map_or(true, |l|
                            item.location.to_lowercase().contains(&l.to_lowercase())
                        ) &&
                    description.as_ref().map_or(true, |d| description_match(d, &item.description))
            })
            .map(|(_, item)| item)
            .collect();

        results.sort_by(|a, b| b.date.cmp(&a.date)); // Sort by date, most recent first
        return Ok(results);
    })
}

#[query]
fn get_user_notifications(UserNoty { phone }: UserNoty)->Vec<Notification>{
    user_notifications(phone)
}
#[update]
fn delete_notification(DeleteNoty{notification_id,phone}:DeleteNoty)->Result<String,CustomError>{
    delete_notification_function(notification_id,phone)
}

ic_cdk::export_candid!();
