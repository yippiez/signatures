class Widget {
public:
    Widget();
    Widget(int value);
    Widget(const Widget& other);
    Widget(Widget&& other) noexcept;
    ~Widget();
};
