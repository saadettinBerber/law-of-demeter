package shop.service;

import java.util.function.Consumer;
import shop.domain.Event;

public class Worker {
    private final Consumer<Event> task;

    public Worker(Consumer<Event> task) {
        this.task = task;
    }

    public void run(Event event) {
        task.accept(event);
    }
}
