use law_of_demeter_core::domain::classification::{Classification, Platform};

const VALUE_NAMESPACES: [&str; 1] = ["java.time."];

const VALUES: [&str; 18] = [
    "java.lang.String",
    "java.lang.StringBuilder",
    "java.lang.StringBuffer",
    "java.lang.CharSequence",
    "java.lang.Boolean",
    "java.lang.Byte",
    "java.lang.Character",
    "java.lang.Short",
    "java.lang.Integer",
    "java.lang.Long",
    "java.lang.Float",
    "java.lang.Double",
    "java.lang.Number",
    "java.lang.Class",
    "java.math.BigDecimal",
    "java.math.BigInteger",
    "java.util.UUID",
    "java.util.Date",
];

const CONTAINER_NAMESPACES: [&str; 1] = ["java.util.stream."];

const CONTAINERS: [&str; 38] = [
    "java.lang.Iterable",
    "java.util.Collection",
    "java.util.List",
    "java.util.ArrayList",
    "java.util.LinkedList",
    "java.util.Vector",
    "java.util.Stack",
    "java.util.Set",
    "java.util.HashSet",
    "java.util.LinkedHashSet",
    "java.util.SortedSet",
    "java.util.NavigableSet",
    "java.util.TreeSet",
    "java.util.EnumSet",
    "java.util.Queue",
    "java.util.Deque",
    "java.util.ArrayDeque",
    "java.util.PriorityQueue",
    "java.util.Map",
    "java.util.HashMap",
    "java.util.LinkedHashMap",
    "java.util.SortedMap",
    "java.util.NavigableMap",
    "java.util.TreeMap",
    "java.util.EnumMap",
    "java.util.Hashtable",
    "java.util.Iterator",
    "java.util.ListIterator",
    "java.util.Optional",
    "java.util.OptionalInt",
    "java.util.OptionalLong",
    "java.util.OptionalDouble",
    "java.util.concurrent.ConcurrentMap",
    "java.util.concurrent.ConcurrentHashMap",
    "java.util.concurrent.CopyOnWriteArrayList",
    "java.util.concurrent.BlockingQueue",
    "java.util.concurrent.LinkedBlockingQueue",
    "java.util.concurrent.ConcurrentLinkedQueue",
];

const DATA_STRUCTURES: [&str; 3] = [
    "java.util.Map$Entry",
    "java.util.AbstractMap$SimpleEntry",
    "java.util.AbstractMap$SimpleImmutableEntry",
];

const BEHAVIORLESS_SUPERTYPES: [&str; 6] = [
    "java.lang.Object",
    "java.lang.Record",
    "java.lang.Enum",
    "java.io.Serializable",
    "java.lang.Cloneable",
    "java.lang.Comparable",
];

pub fn java_platform() -> Platform {
    let types = [
        (&VALUES[..], Classification::Value),
        (&CONTAINERS[..], Classification::Container),
        (&DATA_STRUCTURES[..], Classification::DataStructure),
    ];
    let namespaces = [
        (&VALUE_NAMESPACES[..], Classification::Value),
        (&CONTAINER_NAMESPACES[..], Classification::Container),
    ];
    let platform = types
        .into_iter()
        .flat_map(|(names, classification)| names.iter().map(move |name| (name, classification)))
        .fold(Platform::new(), |platform, (name, classification)| {
            platform.with_type(name, classification)
        });
    let platform = namespaces
        .into_iter()
        .flat_map(|(prefixes, classification)| {
            prefixes.iter().map(move |prefix| (prefix, classification))
        })
        .fold(platform, |platform, (prefix, classification)| {
            platform.with_namespace(prefix, classification)
        });
    BEHAVIORLESS_SUPERTYPES
        .iter()
        .fold(platform, |platform, name| {
            platform.with_behaviorless_supertype(name)
        })
        .with_root_supertype("java.lang.Object")
        .with_object_method("equals", 1)
        .with_object_method("hashCode", 0)
        .with_object_method("toString", 0)
        .with_object_method("canEqual", 1)
}
