package shop;

public class Customer {
    private String name;
    private String surname;

    public String getName() {
        return name;
    }

    public String getFullName() {
        return name + " " + surname;
    }
}
