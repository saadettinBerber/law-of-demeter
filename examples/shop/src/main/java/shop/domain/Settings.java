package shop.domain;

public class Settings {
    private final Gateway gateway = new Gateway();

    public Gateway gateway() {
        return gateway;
    }

    public void reload() {
    }
}
