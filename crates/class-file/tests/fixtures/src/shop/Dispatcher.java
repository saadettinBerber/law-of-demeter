package shop;

import java.util.function.Consumer;

public class Dispatcher<T> {
    public Dispatcher(Consumer<T> handler) {
    }

    public Dispatcher(String name, Runnable... tasks) {
    }
}
