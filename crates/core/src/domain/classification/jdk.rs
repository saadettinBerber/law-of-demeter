use super::Classification;

const VALUE_PACKAGES: [&str; 1] = ["java.time."];

const VALUES: [&str; 17] = [
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
    "java.math.BigDecimal",
    "java.math.BigInteger",
    "java.util.UUID",
    "java.util.Date",
];

const CONTAINER_PACKAGES: [&str; 1] = ["java.util.stream."];

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

pub(super) fn is_behaviorless(name: &str) -> bool {
    BEHAVIORLESS_SUPERTYPES.contains(&name)
}

pub(super) fn classify(name: &str) -> Option<Classification> {
    if DATA_STRUCTURES.contains(&name) {
        Some(Classification::DataStructure)
    } else if is_listed(name, &VALUES, &VALUE_PACKAGES) {
        Some(Classification::Value)
    } else if is_listed(name, &CONTAINERS, &CONTAINER_PACKAGES) {
        Some(Classification::Container)
    } else {
        None
    }
}

fn is_listed(name: &str, names: &[&str], packages: &[&str]) -> bool {
    names.contains(&name) || packages.iter().any(|package| name.starts_with(package))
}
