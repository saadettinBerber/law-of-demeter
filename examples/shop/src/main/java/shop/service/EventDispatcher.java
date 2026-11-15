package shop.service;

import java.util.Map;
import java.util.function.Consumer;
import shop.domain.Event;

public class EventDispatcher {
    private final Map<String, Consumer<Event>> handlers;

    public EventDispatcher() {
        handlers = Map.of("created", e -> e.getReceipt().close());
    }

    public void dispatch(String type, Event event) {
        handlers.get(type).accept(event);
    }
}
