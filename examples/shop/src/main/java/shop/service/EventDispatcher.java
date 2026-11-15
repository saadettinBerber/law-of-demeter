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

    private Consumer<Event> onShipped;

    public void listen(Event event) {
        Consumer<Event> handler = e -> e.getReceipt().getOrder().ship();
        handler.accept(event);
        onShipped = e -> e.getReceipt().getOrder().ship();
    }

    public Consumer<Event> shipper() {
        return e -> e.getReceipt().getOrder().ship();
    }

    public Worker worker() {
        return new Worker(e -> e.getReceipt().getOrder().ship());
    }

    public void listenTyped() {
        Consumer<Event> typed = (Event e) -> e.getReceipt().getOrder().ship();
    }
}
