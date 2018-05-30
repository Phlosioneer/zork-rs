
use ffi;
use ffi::objects::ObjectEntry;

#[derive(Debug, Clone)]
pub enum SearchError {
    NoMatches,
    MultipleMatches(Vec<usize>)
}

fn search_room(noun: usize, adjective: usize,
               special_object: usize, room: usize) -> Result<usize, SearchError> {
    trace!("Redirecting schlst_ to room search...");
    let room_filter = |object: &ObjectEntry| object.is_in_room(room);
    search_objects(noun, adjective, special_object, room_filter)
}

fn search_container(noun: usize, adjective: usize, 
                    special_object: usize, container: usize) -> Result<usize, SearchError> {
    trace!("Redirecting schlst_ to container search...");
    let container_filter = |object: &ObjectEntry| object.container == Some(container);
    search_objects(noun, adjective, special_object, container_filter)
}

fn search_adventurer(noun: c_int, adjective: c_int,
                     special_object: usize, adventurer: c_int) -> Result<usize, SearchError> {
    trace!("Redirecting schlst_ to adventurer search...");
    let adventurer_filter = |object: &ObjectEntry| object.adventurer == Some(adventurer);
    search_objects(noun, adjective, special_object, adventurer_filter)
}

// Looks through all objects for ones that match the given filter.
fn search_objects<F>(noun: usize, adjective: usize, 
                     special_object: usize, f: F) -> Result<usize, SearchError>
where F: Fn(&ObjectEntry) -> bool
{
    trace!("search_objects({}, {}, {}, F)", noun, adjective, special_object);

    let objects = unsafe { &ffi::objects };

    let filtered_objects: Vec<_> = objects.iter()
        .filter(|object| object.is_visible() && f(&object))
        .collect();
   
    debug!("filtered objects: {:#?}", &filtered_objects);

    // Look for direct matches.
    let mut matches: Vec<_> = filtered_objects.iter()
        .filter(|object| object.get_id() == special_object
                || object.matches(noun as usize, adjective as usize))
        .map(ObjectEntry::get_id)
        .collect();

    debug!("direct matches: {:?}", &matches);

    // Look for indirect matches.
    let mut indirect_matches: Vec<_> = filtered_objects.into_iter()
        .filter(|object| object.is_open() 
                || object.is_transparent()
                || object.is_searchable())
        .map(|object| search_container(noun, adjective, special_object, object.get_id()))
        .flat_map(|result| match result {
            Ok(id) => vec![id],
            Err(SearchError::NoMatches) => vec![],
            Err(SearchError::MultipleMatches(matches)) => matches
        })
        .collect();

    debug!("indirect matches: {:?}", &indirect_matches);

    // Collect all the matches together.
    matches.append(&mut indirect_matches);

    if matches.len() == 0 {
        trace!("search_objects: No objects found.");
        Err(SearchError::NoMatches)
    } else if matches.len() == 1 {
        trace!("search_objects: Returning {}", matches[0]);
        Ok(matches[0])
    } else {
        trace!("search_objects: Multiple matches found: {:?}", matches);
        Err(SearchError::MultipleMatches(matches[0]))
    }
}

