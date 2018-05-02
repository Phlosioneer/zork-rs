
// This is the vocab that used to be within parse.h.
//
// This is an overview of how Zork used to parse commands.
//
// First, the input string was given to a lexer. The lexer skipped all space
// characters, then tried to parse a token.
//
// To parse a token, every character was turned into a number. Characters A..Z
// mapped to 1..26, the - character was 27, and the numbers 1..9 mapped to 31..39.
// (Curiously, there was no number 0.) A space ended the token. The special
// characters '.' and ',' would split a command string into multiple commands,
// each parsed consecutively, until the end of the input was found. Any other
// character was an error and aborted parsing immediately.
//
// The characters were then combined three-at-a-time into an unsigned 16-bit
// integer, with the following formula:
// 
// token = first * 1600 + second * 40 + third
//
// This system was called "Radix-50", even though it's actually an encoding in
// base 40.
//
// Words were truncated to 6 characters; any characters after that were ignored.
// Therefore, each word was two consecutive tokens. Any missing characters simply
// had the value 0.
//
// The array of tokens was then given to the string parser. Input lines were read
// 79 characters at a time, and the smallest token was 2 characters long (a letter
// and a space), which means there were at most 40 words. However, the token
// array only had space for 40 tokens, while the maximum 40 words would require
// 80 tokens. The token array was allocated on the stack, and was not bounds checked,
// so inputting too many tokens would cause memory corruption, and possibly later
// a segfault.
//
// The string parser iterated through every pair of tokens, which represented
// one word, and tried to find it in its vocab arrays. Throughout this description,
// "word" and "pair of tokens" will be used interchangeably.
//
// There were 6 vocab arrays, listed here in order of priority:
// - Buzzwords
// - Verbs
// - Directions
// - Prepositions
// - Adjectives
// - Objects
// 
// The buzzword array was encoded as an array of words. If the token pair matched
// a word in the array, then the buzzword was identified by the index of that word
// (divided by two, because words are pairs of numbers). The array was null-
// terminated (two 0's, making a null word).
//

struct Buzzword {
    name: &'static str,
    id: usize
}

impl Buzzword {
    pub fn new(name: &'static str, id: usize) {
        Buzzword { name, id }
    }
}

lazy_static! {
    static BUZZWORDS: Vec<Buzzword> = vec![
        Buzzword::new("and", 0),
        Buzzword::new("by", 1),
        Buzzword::new("is", 2),
        Buzzword::new("a", 3),
        Buzzword::new("an", 4),
        Buzzword::new("the", 5),
        Buzzword::new("run", 6),
        Buzzword::new("go", 7),
        Buzzword::new("proceed", 8)
    ];
}

// 
// The verb array was more complicated. Each verb was stored as a list of words,
// that were synonyms. Then, after the words, there were a series of numbers to
// specify the syntax of the verb, called "syntax slots". The first syntax slot
// was guaranteed to be less than 1600, which is the minimum value for a token
// (because the first character cannot be 0 and is multiplied by 1600).
//
// The first syntax slot was a count of how many sytax slots there are, NOT
// including this one. This is so that the parser can quickly skip over the
// syntax slots to the next word.
// 
// The second syntax slot was a combination of 5 flag bits and a verb ID number.
// The flags were:
// - Bit 14, true if the syntax includes a direct object.
// - Bit 13, true if the syntax includes an indirect object.
// - Bit 12, true if the direct object is implicit
// - Bit 11, true if the indirect object comes before the direct object. If false,
//   the direct object comes first. // TODO: Confirm this!
// - Bit 10, true if "this is the default syntax for orphanery" // TODO: ???
// The verb ID number was stored in the lower 8 bits.
// 
// The rest of the syntax slots were taken by "object descriptions". There were
// 0, 1, or 2 of them, depending on the direct and indirect object flags. // TODO: what order?
// Object descriptions are three pairs of flags and preposition numbers. // TODO: More info.
// The flags were:
// - Bit 14, true if the game should search the adventurer's inventory for the
//   object.
// - Bit 13, true if the game should search the room for the object.
// - Bit 12, true if the game should try to take / consume the object.
// - Bit 11, true if the object MUST be present.
// - Bit 10, "Qualifying bits (normally -1, -1) are same as FWIM bits" // TODO: ???
// - Bit 9, true if the object must be reachable.
// The preposition ID number was stored in the lower 8 bits.
//
// Unused parts of the object descriptions were replaced with the value -1.Also,
// an object description can be cut short by the length given in the first syntax
// slot.
//
// // TODO: This isn't the whole story - one of the entries has 22 things!
//
// The verb array was terminated with a -1 entry.
// 

struct Verb {
    names: Vec<&'static str>,
    id: usize,
    
    has_direct_object: bool,
    has_indirect_object: bool,
    direct_is_implicit: Option<bool>,
    indirect_first: Option<bool>,
    orphanery: bool,

    direct_object: Option<VerbObject>,
    indirect_object: Option<VerbObject>
}

struct VerbObject {
    prep_id: usize,

    search_player: bool,
    search_room: bool,
    must_have_item: bool,
    must_be_reachable: bool,

    qualifying_bits: bool,
}

impl Verb {
    pub fn new_simple(name: &'static str, id: usize) -> Verb {
        Verb {
            names: vec![name],
            id,

            has_direct_object: false,
            has_indirect_object: false,
            direct_is_implicit: None,
            indirect_first: None,
            orphanery: false,

            direct_object: None,
            indirect_object: None
        }
    }

    pub fn with_names(names: Vec<&'static str>, id: usize) -> Verb {
        Verb {
            names,
            id,

            has_direct_object: false,
            has_indirect_object: false,
            direct_is_implicit: None,
            indirect_first: None,
            orphanery: false,

            direct_object: None,
            indirect_object: None
        }
    }
}
/*
lazy_static! {
    static Verbs: Vec<Verb> = vec![
        Verb::new_simple("brief", 70),
        Verb::new_simple("verbose", 71),
        Verb::new_simple("superb", 72), // Super-brief?
        Verb::new_simple("stay", 73),
        Verb::new_simple("version", 74),
        Verb::with_names(vec!["swim", "bathe", "wade"], 75),
        Verb::new_simple("geronimo", 76),
        Verb::with_names(vec!["Ulysses", "Odyssey"], 77),
        Verb::new_simple("well", 78),
        Verb::new_simple("pray", 79),
        Verb::new_simple("treasure", 80),
        Verb::new_simple("temple", 81),
        Verb::new_simple("blast", 82),
        Verb::new_simple("score", 83),
        Verb::with_names(vec!["q", "quit"], 84),
        Verb::new_simple("help", 40),
        Verb::new_simple("info", 41),
        Verb::with_names(vec!["history", "update"], 42),
        Verb::new_simple("back", 43),
        Verb::with_names(vec!["sigh", "mumble"], 44),
        Verb::with_names(vec!["chomp", "lose", "barf"], 45),
        Verb::new_simple("dungeon", 46),
        Verb::new_simple("froboz", 47),
        Verb::with_names(vec!["foo", "bletch", "bar"], 48),
        Verb::new_simple("repent", 49),
        Verb::with_names(vec!["hours", "schedule"], 50),
        Verb::new_simple("win", 51),
        Verb::with_names(vec!["yell", "scream", "shout"], 52),
        Verb::with_names(vec!["hop", "skip"], 53),
        Verb::with_names(vec!["fuck", "shit", "damn", "curse"], 54),
        Verb::new_simple("zork", 55),
        Verb {
            names: vec!["granite"],
            id: 56,

            has_direct_object: true,
            has_indirect_object: false,
            direct_is_implicit: Some(true),
            indirect_first: None,
            
            orphanery: false,

            direct_object: None,
            indirect_object: None
        },
        Verb::new_simple("save", 149),
        Verb::new_simple("restore", 150),
        Verb::new_simple("time", 90),
        Verb::new_simple("diagno", 94),     // TODO: ??
        Verb::with_names(vec!["exorcism", "exorcise"], 105),
        Verb::with_names(vec!["i", "inventory"], 133),
        Verb::new_simple("wait", 128),
        Verb::with_names(vec!["incant", "incantation"], 95),
        Verb::new_simple("answer", 96),
        Verb::new_simple("again", 57),
        Verb::new_simple("noobj", 58),
        Verb::with_names(vec!["bug", "gripe", "complain"], 59),
        Verb::with_names(vec!["feature", "comment", "suggest", "idea"], 60),
        Verb::new_simple("room", 65),
        Verb::new_simple("object", 66),
        Verb::with_names(vec!["rname", "rename"], 67),
        Verb {
            names: vec!["deflate"], 
            id: 103,

            has_direct_object: true,
            has_indirect_object: false,
            direct_is_implicit: Some(true),
            indirect_first: None,
            orphanery: false,

            direct_object: None,
            indirect_object: None
        },
        Verb {
            names: vec!["describe", "what", "examine"],
            id: 120,

            has_direct_object: true,
            has_indirect_object: false,
            direct_is_implicit: Some(true),
            indirect_first: None,
            orphanery: false,

            direct_object: None,
            indirect_object: None
        },
        Verb {
            names: vec!["fill"],
            id: 134,

            has_direct_object: true,
            has_indirect_object: true,
            direct_is_implicit: Some(false),
            indirect_first: Some(false),
            orphanery: false,

            direct_object: Some(vec![
                VerbObject {
                    prep_id: 0,
                    
                    search_player: true,
                    search_room: true,
                    must_have_item: false,
                    must_be_reachable: true,

                    qualifying_bits: false
                },
                VerbObject {
                    prep_id: 128,

                    search_player: false,
                    search_room: false,
                    must_have_item: false,
                    must_be_reachable: false,

                    qualifying_bits: false
                },
                VerbObject {
                    prep_id: 0,
                    
                    search_player: false,

    ];
}


*/

//
// The directions array is composed of a single token, followed by an id number.
// The numbers were stored shifted right by 10. // TODO: Why?


struct Direction {
    names: Vec<&'static str>,
    id: usize
}

impl Direction {
    pub fn new(names: Vec<&'static str>, bits: Vec<u8>) -> Direction {
        Direction { names, bits }
    }
}

lazy_static! {
    static Directions: Vec<Direction> = vec![
        Direction::new(
            vec!["n", "north"],
            1
        ),
        Direction::new(
            vec!["ne", "northeast", "north-east"],
            2
        ),
        Direction::new(
            vec!["e", "east"],
            3
        ),
        Direction::new(
            vec!["se", "southeast", "south-east"],
            4
        ),
        Direction::new(
            vec!["s", "south"],
            5
        ),
        Direction::new(
            vec!["sw", "southwest", "south-west"],
            6
        ),
        Direction::new(
            vec!["w", "west"],
            7
        ),
        Direction::new(
            vec!["nw", "northwest", "north-west"],
            8
        ),
        Direction::new(
            vec!["u", "up"],
            9
        ),
        Direction::new(
            vec!["d", "down"],
            10
        ),
        Direction::new(
            vec!["launch"],
            11
        ),
        Direction::new(
            vec!["land"],
            12
        ),
        Direction::new(
            vec!["in"],
            13
        )
        Direction::new(
            vec!["enter"],
            14
        ),
        Direction::new(
            vec!["exit", "out", "leave", "cross"],
            15
        ),
    ];
}

//
// The preposition array was stored as a list of one or more words, followed by
// an id number. The id numbers could be differentiated from the words by their
// size; the smallest token was 1600, while the maximum allowed id was 255.
//

struct Preposition {
    names: Vec<&'static str>,
    id: usize
}

impl Preposition {
    pub fn new(name: &'static str, id: usize) -> Preposition {
        Preposition { 
            names: vec![name], 
            id 
        }
    }

    pub fn with_names(names: &'static str, id: usize) -> Preposition {
        Preposition { names, id}
    }
}

lazy_static! {
    static Prepositions: Vec<Preposition> = vec![
        Preposition::new("over", 1),
        Preposition::with_names(vec!["with", "using", "through"], 2),
        Preposition::new("at", 3),
        Preposition::new("to", 4),
        Preposition::with_names(vec!["in", "inside", "into"], 5),
        Preposition::new("down", 6),
        Preposition::new("up", 7),
        Preposition::new("under", 8),
        Preposition::new("of", 9),
        Preposition::new("on", 10),
        Preposition::new("off", 11)
    ];
}

//
// The adjectives array was stored as a word followed by any number of object id's
// that the adjective applies to. Id's could be differentiated from words because
// the smallest value for a token was 1600, while the largest allowed id was 255.
//
// The objects array was stored the same way as the adjectives array.
//

struct Object {
    id: usize,
    
    names: Vec<&'static str>,
    adjectives: Vec<&'static str>,
}

lazy_static! {
    static OBJECTS: Vec<Object> = vec![
        Object {
            id: 1,

            names: vec!["bag", "sack"],
            adjectives: vec!["brown", "elongated"]
        },
        Object {
            id: 2,

            names: vec!["garlic", "clove"],
            adjectives: vec![]
        },
        Object {
            id: 3,

            names: vec!["food", "sandwitch", "lunch", "dinner"],
            adjectives: vec!["hot", "pepper"]
        },
        Object {
            id: 4,

            names: vec!["gunk", "piece", "slag"],
            adjectives: vec!["vitreous"]
        },
        Object {
            id: 5,

            names: vec!["coal", "pile", "heap"],
            adjectives: vec!["small"]
        },
        Object {
            id: 6,

            names: vec!["figurine"],
            adjectives: vec!["jade"]
        },
        Object {
            id: 7,

            names: vec!["machine", "pdpnm", "pdpnn", "dryer", "lid"],
            adjectives: vec![]
        },
        Object {
            id: 8,

            names: vec!["diamond"],
            adjectives: vec!["huge", "enormous"]
        },
        Object {
            id: 9,

            names: vec!["case"],
            adjectives: vec!["trophy"]
        },
        Object {
            id: 10,

            names: vec!["bottle", "container"],
            adjectives: vec!["clear", "glass"]
        },
        Object {
            id: 11,

            names: vec!["water", "quantity", "liquid", "hoo"],
            adjectives:
        },
        Object {
            id: 12,

            names: vec!["rope", "hemp", "coil"],
            adjectives: vec!["large"]
        },
        Object {
            id: 13,

            names: vec!["knife", "blade"],
            adjectives: vec!["nasty"]
        },
        Object {
            id: 14,

            names: vec!["sword", "orchrist", "glamring", "blade"],
            adjectives: vec!["elvish"]
        },
        Object {
            id: 15,

            names: vec!["lamp", "lantern"],
            adjectives: vec!["brass"]
        },
        Object {
            id: 16,

            names: vec!["lamp", "lantern"],
            adjectives: vec!["brass", "broken"]
        },
        Object {
            id: 17,

            names: vec!["rug", "carpet"],
            adjectives: vec!["oriental"]
        },
        Object {
            id: 18,

            names: vec!["leaves", "leaf"],
            adjectives:
        },
        Object {
            id: 19,

            names: vec!["troll"],
            adjectives:
        },
        Object {
            id: 20,

            names: vec!["axe"],
            adjectives: vec!["bloody"]
        },
        Object {
            id: 21,
            
            names: vec!["knife"],
            adjectives: vec!["rusty"]
        },
        Object {
            id: 22,

            names: vec!["lamp", "lantern"],
            adjectives: vec!["broken", "burned", "dead"]
        },
        Object {
            id: 23,

            names: vec!["keys", "key", "set"],
            adjectives:
        },
        Object {
            id: 24,

            names: vec!["bones", "skeleton", "body"],
            adjectives:
        },
        Object {
            id: 25,

            names: vec!["coins", "bag"],
            adjectives: vec!["old", "leather"]
        },
        Object {
            id: 26,

            names: vec!["bar"],
            adjectives: vec!["large", "platinum"]
        },
        Object {
            id: 27,

            names: vec!["necklace", "pearls"],
            adjectives: vec!["pearl"]
        },
        Object {
            id: 28,

            names: vec!["mirror"],
            adjectives: vec![]
        },
        Object {
            id: 29,

            names: vec!["mirror"],
            adjectives: vec![]
        },
        Object {
            id: 30,

            names: vec!["ice", "mass", "glacier"],
            adjectives: vec![]
        },
        Object {
            id: 31,

            names: vec!["ruby"],
            adjectives: vec!["moby"]
        },
        Object {
            id: 32,

            names: vec!["trident", "fork"],
            adjectives: vec!["crystal"]
        },
        Object {
            id: 33,

            names: vec!["coffin", "casket"],
            adjectives: vec!["gold"]
        },
        Object {
            id: 34,

            names: vec!["torch"],
            adjectives: vec!["ivory"]
        },
        Object {
            id: 35,

            names: vec!["cage", "dumbwaiter", "basket"],
            adjectives: vec![]
        },
        Object {
            id: 36,

            names: vec!["cage", "dumbwaiter", "basket"],
            adjectives: vec![]
        },
        Object {
            id: 37,

            names: vec!["bracelet", "jewel"],
            adjectives: vec!["sapphire"]
        },
        Object {
            id: 38,

            names: vec!["timber", "pile"],
            adjectives: vec!["wooden", "wood"]
        },
        Object {
            id: 39,

            names: vec!["box"],
            adjectives: vec!["steel", "dented"]
        },
        Object {
            id: 40,

            names: vec!["stradivarius", "violin"],
            adjectives: vec!["fancy"]
        },
        Object {
            id: 41,

            names: vec!["engraving", "inscription"],
            adjectives: vec!["old", "ancient"]
        },
        Object {
            id: 42,

            names: vec!["ghost", "spirit", "fiend"],
            adjectives: vec![]
        },
        Object {
            id: 43,
            
            names: vec!["grail"],
            adjectives: vec![]
        },
        Object {
            id: 44,

            names: vec!["prayer", "inscription"],
            adjectives: vec!["old", "ancient"]
        },
        Object {
            id: 45,

            names: vec!["trunk", "chest"],
            adjectives: vec!["old"]
        },
        Object {
            id: 46,

            names: vec!["bell"],
            adjectives: vec!["brass", "small"]
        },
        Object {
            id: 47,

            names: vec!["book", "bible", "goodbook", "prayerbook"],
            adjectives: vec!["large", "black"]
        },
        Object {
            id: 48,

            names: vec!["candle", "pair"],
            adjectives: vec![]
        },
        Object {
            id: 49,

            names: vec!["guidebook", "guide", "book"],
            adjectives: vec!["tour"]
        },
        Object {
            id: 50,

            names: vec!["paper", "newspaper", "issue", "report", "magazine", "news"],
            adjectives: vec![]
        },
        Object {
            id: 51,

            names: vec!["matchbox", "match", "matches"],
            adjectives: vec![]
        },
        Object {
            id: 52,

            names: vec!["advertizement", "pamphlet", "leaflet", "booklet"],
            adjectives: vec!["small"]
        },
        Object {
            id: 53,

            names: vec!["mailbox", "box"],
            adjectives: vec!["small"]
        },
        Object {
            id: 54,

            names: vec!["tube", "toothpaste"],
            adjectives: vec![]
        },
        Object {
            id: 55,

            names: vec!["putty", "material", "glue", "gunk"],
            adjectives: vec!["viscous"]
        },
        Object {
            id: 56,

            names: vec!["wrench"],
            adjectives: vec![]
        },
        Object {
            id: 57,

            names: vec!["screwdriver"],
            adjectives: vec![]
        },
        Object {
            id: 58,

            names: vec!["cyclopse", "monster"],
            adjectives: vec![]
        },
        Object {
            id: 59,

            names: vec!["chalice", "cup", "goblet"],
            adjectives: vec![]
        },
        Object {
            id: 60,

            names: vec!["painting", "art", "cantur", "picture", "work", "masterpiece"],
            adjectives: vec![]
        },
        Object {
            id: 61,

            names: vec!["thief", "robber", "criminal", "bandit", "crook", "gent",
                "gentleman", "man", "thug", "bagman"],
            adjectives: vec![]
        },
        Object {
            id: 62,

            names: vec!["stille"],
            adjectives: vec!["vicious"]
        },
        Object {
            id: 63,

            names: vec!["window"],
            adjectives: vec![]
        },
        Object {
            id: 64,

            names: vec!["bolt", "nut"],
            adjectives: vec![]
        },
        Object {
            id: 65,

            names: vec!["grate", "grating"],
            adjectives: vec![]
        },
        Object {
            id: 66,

            names: vec!["door", "trapdoor", "trap-door"],
            adjectives: vec!["trap"]
        },
        Object {
            id: 67,

            names: vec!["letter", "door"],
            adjectives: vec![]
        },
        Object {
            id: 68,

            names: vec!["door"],
            adjectives: vec!["front"]
        },
        Object {
            id: 69,

            names: vec!["door"],
            adjectives: vec!["stone"]
        },
        Object {
            id: 70,

            names: vec!["switch"],
            adjectives: vec![]
        },
        Object {
            id: 71,

            names: vec!["head"],
            adjectives: vec![]
        },
        Object {
            id: 72,
            
            names: vec!["corpse"],
            adjectives: vec!["mangled"]
        },
        Object {
            id: 73,

            names: vec!["bodies", "body", "corpse", "corpses"],
            adjectives: vec![]
        },
        Object {
            id: 74,

            names: vec!["dam", "gates", "gate", "fcd"],
            adjectives: vec![]
        },
        Object {
            id: 75,

            names: vec!["rail", "railing"],
            adjectives: vec![]
        },
        Object {
            id: 76,

            names: vec!["button", "switch", "gates", "gate"],
            adjectives: vec![]
        },
        Object {
            id: 77,

            names: vec!["bubble"],
            adjectives: vec![]
        },
        Object {
            id: 78,

            names: vec!["leak", "drip", "hole", "pile"],
            adjectives: vec![]
        },
        Object {
            id: 79,

            names: vec!["switch", "button"],
            adjectives: vec!["red"]
        },
        Object {
            id: 80,

            names: vec!["switch", "button"],
            adjectives: vec!["yellow"]
        },
        Object {
            id: 81,

            names: vec!["switch", "button"],
            adjectives: vec!["brown"]
        },
        Object {
            id: 82,

            names: vec!["switch", "button"],
            adjectives: vec!["blue"] 
        },
        Object {
            id: 83,

            names: vec!["bat"],
            adjectives: vec!["vampire", "vampiric"]
        },
        Object {
            id: 84,

            names: vec!["rainbow"],
            adjectives:
        },
        Object {
            id: 85,

            names: vec!["pot"],
            adjectives: vec!["gold"]
        },
        Object {
            id: 86,

            names: vec!["statue", "sculpture", "rock"],
            adjectives: vec![]
        },
        Object {
            id: 87,
            
            names: vec!["boat", "plastic", "pile"],
            adjectives: vec![]
        },
        Object {
            id: 88,

            names: vec!["pile", "boat", "plastic"],
            adjectives: vec![]
        },
        Object {
            id: 89,

            names: vec!["pump", "airpump", "air-pump"],
            adjectives: vec!["small"]
        },
        Object {
            id: 90,
            
            names: vec!["boat"],
            adjectives: vec!["magic", "seaworthy"]
        },
        Object {
            id: 91,

            names: vec!["label", "fineprint"],
            adjectives: vec!["tan"]
        },
        Object {
            id: 92,

            names: vec!["stick"],
            adjectives: vec!["broken", "sharp"]
        },
        Object {
            id: 93,

            names: vec!["barrel"],
            adjectives: vec![]
        },
        Object {
            id: 94,

            names: vec!["buoy"],
            adjectives: vec!["red"]
        },
        Object {
            id: 95,

            names: vec!["emerald"],
            adjectives: vec!["large"]
        },
        Object {
            id: 96,

            names: vec!["shovel"],
            adjectives: vec!["large"]
        },
        Object {
            id: 97,

            names: vec!["guano", "crap", "shit", "hunk"],
            adjectives: vec![]
        },
        Object {
            id: 98,

            names: vec!["balloon", "basket"],
            adjectives: vec!["wicker"]
        },
        Object {
            id: 99,

            names: vec!["reception"],
            adjectives: vec![]
        },
        Object {
            id: 100,

            names: vec!["bag"],
            adjectives: vec!["cloth"]
        },
        Object {
            id: 101,

            names: vec!["wire", "rope"],
            adjectives: vec!["braided"]
        },
        Object {
            id: 102,

            names: vec!["hook"],
            adjectives: vec!["small"]
        },
        Object {
            id: 103,

            names: vec!["hook"],
            adjectives: vec!["small"]
        },
        Object {
            id: 104,

            names: vec!["zorkmi", "coin"],
            adjectives: vec!["gold"]
        },
        Object {
            id: 105,

            names: vec!["safe", "box"],
            adjectives: vec![]
        },
        Object {
            id: 106,

            names: vec!["card", "note"],
            adjectives: vec![]
        },
        Object {
            id: 107,

            names: vec!["slot", "hole"],
            adjectives: vec![]
        },
        Object {
            id: 108,

            names: vec!["crown"],
            adjectives: vec!["gaudy"]
        },
        Object {
            id: 109,
            
            names: vec!["brick"],
            adjectives: vec!["square", "clay"]
        },
        Object {
            id: 110,

            names: vec!["fuse", "coil", "wire"],
            adjectives: vec!["shiny", "thin"]
        },
        Object {
            id: 111,

            names: vec!["gnome"],
            adjectives: vec![]
        },
        Object {
            id: 112,

            names: vec!["label", "pile"],
            adjectives: vec!["blue"]
        },
        Object {
            id: 113,

            names: vec!["balloon", "basket"],
            adjectives: vec!["broken"]
        },
        Object {
            id: 114,

            names: vec!["book"],
            adjectives: vec!["blue"]
        },
        Object {
            id: 115,

            names: vec!["book"],
            adjectives: vec!["green"]
        },
        Object {
            id: 116,

            names: vec!["book"],
            adjectives: vec!["purple"]
        },
        Object {
            id: 117,

            names: vec!["book"],
            adjectives: vec!["white"]
        },
        Object {
            id: 118,

            names: vec!["stamp"],
            adjectives: vec![]
        },
        Object {
            id: 119,

            names: vec!["tomb", "crypt", "grave"],
            adjectives: vec!["marble"]
        },
        Object {
            id: 120,

            names: vec!["heads", "poles", "implements", "losers", "head"],
            adjectives: vec![]
        },
        Object {
            id: 121,

            names: vec!["cokes", "bottle", "bottles"],
            adjectives: vec!["coke", "empty"] 
        },
        Object {
            id: 122,

            names: vec!["listing", "stack", "printout", "paper"],
            adjectives: vec!["enormous"]
        },
        Object {
            id: 123,

            names: vec!["case"],
            adjectives: vec!["large"]
        },
        Object {
            id: 124,

            names: vec!["cage"],
            adjectives: vec![]
        },
        Object {
            id: 125,

            names: vec!["cage"],
            adjectives: vec!["steel"]
        },
        Object {
            id: 126,

            names: vec!["sphere", "ball"],
            adjectives: vec!["glass"]
        },
        Object {
            id: 127,

            names: vec!["button"],
            adjectives: vec!["square"]
        },
        Object {
            id: 128,

            names: vec!["button"],
            adjectives: vec!["round"]
        },
        Object {
            id: 129,

            names: vec!["button"],
            adjectives: vec!["triangle", "triangular"]
        },
        Object {
            id: 130,

            names: vec!["etching", "walls", "wall"],
            adjectives: vec![]
        },
        Object {
            id: 131,

            names: vec!["etching", "walls", "wall"],
            adjectives: vec![]
        },
        Object {
            id: 132,

            names: vec!["flask"],
            adjectives: vec!["glass"]
        },
        Object {
            id: 133,

            names: vec!["pool", "sewage"],
            adjectives: vec!["large"]
        },
        Object {
            id: 134,

            names: vec!["tin", "saffron", "spices"],
            adjectives: vec!["rare"]
        },
        Object {
            id: 135,

            names: vec!["table"],
            adjectives: vec!["large", "oblong"]
        },
        Object {
            id: 136,

            names: vec!["post", "posts"],
            adjectives: vec!["wooden", "wood"]
        },
        Object {
            id: 137,

            names: vec!["bucket"],
            adjectives: vec!["wooden", "wood"]
        },
        Object {
            id: 138,
            
            names: vec!["cake"],
            adjectives: vec!["eat-me", "eatme"]
        },
        Object {
            id: 139,
            
            names: vec!["icing", "cake"],
            adjectives: vec!["orange"]
        },
        Object {
            id: 140,

            names: vec!["cake", "icing"],
            adjectives: vec!["red"]
        },
        Object {
            id: 141,

            names: vec!["cake", "icing"],
            adjectives: vec!["blue", "ecch"]
        },
        Object {
            id: 142,

            names: vec!["robot", "robby", "cppo", "rudo"],
            adjectives: vec![]
        },
        Object {
            id: 143,

            names: vec!["paper", "piece"],
            adjectives: vec!["green"]
        },
        Object {
            id: 144,

            names: vec!["tree"],
            adjectives: vec![]  // TODO: Shouldn't one or both of these be "large"?
        },
        Object {
            id: 145,

            names: vec!["tree"],
            adjectives: vec![]
        },
        Object {
            id: 146,

            names: vec!["cliff", "ledge"],
            adjectives: vec!["large"] 
        },
        Object {
            id: 147,

            names: vec!["cliff", "ledge"],
            adjectives: vec!["large", "white", "rocky", "sheer"]
        },
        Object {
            id: 148,

            names: vec!["stack", "bills", "zorkmi"],
            adjectives: vec!["omm", "neat"]
        },
        Object {
            id: 149,

            names: vec!["portrait", "painting", "art"],
            adjectives: vec![]
        },
        Object {
            id: 150,

            names: vec!["vault", "cube", "lettering"],
            adjectives: vec!["large", "stone"]
        },
        Object {
            id: 151,

            names: vec!["curtain", "light"],
            adjectives: vec!["shimmering"]
        },
        Object {
            id: 152,
            
            names: vec!["gnome"],
            adjectives: vec![]
        },
        Object {
            id: 153,

            names: vec!["nest"],
            adjectives: vec!["small", "birds"]
        },
        Object {
            id: 154,
            
            names: vec!["egg"],
            adjectives: vec!["birds", "encrusted"]
        },
        Object {
            id: 155,

            names: vec!["egg"],
            adjectives: vec!["broken", "birds", "encrusted"]
        },
        Object {
            id: 156,

            names: vec!["bauble"],
            adjectives: vec!["brass", "beautiful"]
        },
        Object {
            id: 157,

            names: vec!["canary"],
            adjectives: vec!["gold", "clockwork", "mechanical"]
        },
        Object {
            id: 158,

            names: vec!["canary"],
            adjectives: vec!["broken", "gold", "clockwork", "mechanical"]
        },
        Object {
            id: 159,

            names: vec!["panel", "wall"],
            adjectives: vec!["yellow"]
        },
        Object {
            id: 160,

            names: vec!["panel", "wall"],
            adjectives: vec!["white"]
        },
        Object {
            id: 161,

            names: vec!["panel", "wall"],
            adjectives: vec!["red"]
        },
        Object {
            id: 162,

            names: vec!["panel", "wall"],
            adjectives: vec!["black"]
        },
        Object {
            id: 163,

            names: vec!["panel", "wall"],
            adjectives: vec!["mahogany"]
        },
        Object {
            id: 164,

            names: vec!["panel", "wall"],
            adjectives: vec!["pine"]
        },
        Object {
            id: 165,

            names: vec!["bar"],
            adjectives: vec!["wooden", "wood"] 
        },
        Object {
            id: 166,

            names: vec!["pole", "post"],
            adjectives: vec!["long", "center"]
        },
        Object {
            id: 167,

            names: vec!["pole", "post"],
            adjectives: vec!["short"]
        },
        Object {
            id: 168,

            names: vec!["tbar", "t-bar", "bar"],
            adjectives: vec!["t"]
        },
        Object {
            id: 169,

            names: vec!["arrow", "point"],
            adjectives: vec!["compass"]
        },
        Object {
            id: 170,

            names: vec!["switch", "button"],
            adjectives: vec!["red"]
        },
        Object {
            id: 171,

            names: vec!["beam"],
            adjectives: vec!["red"]
        },
        Object {
            id: 172,

            names: vec!["door"],
            adjectives: vec!["bronze"]
        },
        Object {
            id: 173,

            names: vec!["door"],
            adjectives: vec!["wooden", "wood"]
        },
        Object {
            id: 174,

            names: vec!["door"],
            adjectives: vec!["wooden", "wood", "cell", "locked"]
        },
        Object {
            id: 175,

            names: vec!["door"],
            adjectives: vec!["wooden", "wood", "cell"]
        },
        Object {
            id: 176,

            names: vec!["button"],
            adjectives: vec!["large"]
        },
        Object {
            id: 177,

            names: vec!["dial", "sundial"],
            adjectives: vec!["sun"]
        },
        Object {
            id: 178,

            names: vec!["1", "one"],
            adjectives: vec![]
        },
        Object {
            id: 179,

            names: vec!["2", "two"],
            adjectives: vec![]
        },
        Object {
            id: 180,
            
            names: vec!["3", "three"],
            adjectives: vec![]
        },
        Object {
            id: 181,

            names: vec!["4", "four"],
            adjectives: vec![]
        },
        Object {
            id: 182,

            names: vec!["5", "five"],
            adjectives: vec![]
        },
        Object {
            id: 183,

            names: vec!["6", "six"],
            adjectives: vec![]
        },
        Object {
            id: 184,

            names: vec!["7", "seven"],
            adjectives: vec![]
        },
        Object {
            id: 185,

            names: vec!["8", "eight"],
            adjectives: vec![]
        },
        Object {
            id: 186,

            names: vec!["warning", "paper", "piece", "note"],
            adjectives: vec![]
        },
        Object {
            id: 187,

            names: vec!["slit", "slot"],
            adjectives: vec!["small"]
        },
        Object {
            id: 188,

            names: vec!["card"],
            adjectives: vec!["gold"]
        },
        Object {
            id: 189,

            names: vec!["door"],
            adjectives: vec!["steel"]
        },
        // No entries for objects 190 and 191.
        Object {
            id: 192,

            names: vec!["it", "that", "this"],
            adjectives: vec![]
        },
        Object {
            id: 193,

            names: vec!["me", "myself", "cretin"],
            adjectives: vec![]
        },
        Object {
            id: 194,

            names: vec!["all", "everything"],
            adjectives: vec![]
        },
        Object {
            id: 195,

            names: vec!["treasure", "valuable"],
            adjectives: vec![]
        },
        Object {
            id: 196,

            names: vec!["sailor"],
            adjectives: vec![]
        },
        Object {
            id: 197,

            names: vec!["teeth"],
            adjectives: vec![]
        },
        Object {
            id: 198,

            names: vec!["walls", "wall"],
            adjectives: vec![]
        },
        Object {
            id: 199,

            names: vec!["grue"],
            adjectives: vec![]
        },
        Object {
            id: 200,

            names: vec!["hand", "hands"],
            adjectives: vec!["bare"]
        },
        Object {
            id: 201,
            
            names: vec!["lungs", "air"],
            adjectives: vec![]
        },
        Object {
            id: 202,

            names: vec!["aviator", "flyer"],
            adjectives: vec![]
        },
        Object {
            id: 203,

            names: vec!["bird", "songbird"],
            adjectives: vec!["song"]
        },
        Object {
            id: 204,

            names: vec!["tree"],
            adjectives: vec![]
        },
        Object {
            id: 205,

            names: vec!["walls", "wall"],
            adjectives: vec!["north", "northern"]
        },
        Object {
            id: 206,

            names: vec!["walls", "wall"],
            adjectives: vec!["south", "southern"]
        },
        Object {
            id: 207,

            names: vec!["walls", "wall"],
            adjectives: vec!["east", "eastern"]
        },
        Object {
            id: 208,


            names: vec!["walls", "wall"],
            adjectives: vec!["west", "western"]
        },
        Object {
            id: 209,

            names: vec!["water", "quantity", "liquid", "hoo", "h2o"],
            adjectives: vec![]
        },
        Object {
            id: 210,

            names: vec!["guard", "guardian"],
            adjectives: vec![]
        },
        Object {
            id: 211,

            names: vec!["rose"],
            adjectives: vec!["compass"]
        },
        Object {
            id: 212,

            names: vec!["structure", "mirror"],
            adjectives: vec![]
        },
        Object {
            id: 213,

            names: vec!["panel"],
            adjectives: vec![]
        },
        Object {
            id: 214,

            names: vec!["channel"],
            adjectives: vec!["stone"]
        },
        Object {
            id: 215,

            names: vec!["keeper", "masterpiece"],
            adjectives: vec!["dungeon"]
        },
        Object {
            id: 216,

            names: vec!["ladder"],
            adjectives: vec![]
        }
    ];
}










