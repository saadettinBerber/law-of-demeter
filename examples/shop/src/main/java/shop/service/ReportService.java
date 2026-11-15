package shop.service;

import shop.domain.ReportFactory;

public class ReportService {
    public String report(ReportFactory factory) {
        return factory.create().render();
    }

    public String greeting(String name) {
        return new StringBuilder().append("Hello ").append(name).toString();
    }
}
