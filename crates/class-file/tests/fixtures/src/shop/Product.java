package shop;

public class Product extends Entity {
    private String title;
    private long stock;
    private float rating;
    private double price;

    public long identity() {
        return id;
    }

    public String title() {
        return title;
    }

    public void retitle(String title, int revision) {
        this.title = title;
    }

    public String titleAt(int index) {
        return title;
    }

    public void clearTitle(String reason) {
        this.title = null;
    }

    public long getStock() {
        return stock;
    }

    public void setStock(long stock) {
        this.stock = stock;
    }

    public float getRating() {
        return rating;
    }

    public void setRating(float rating) {
        this.rating = rating;
    }

    public double getPrice() {
        return price;
    }

    public void setPrice(double price) {
        this.price = price;
    }

    public static String titleOf(Product product) {
        return product.title;
    }
}
