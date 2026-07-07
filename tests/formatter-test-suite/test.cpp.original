#include <iostream>
#include <vector>
#include <string>
#include   <algorithm>
#include <memory>

// ── CASE 1: Template class — mixed spacing ────────────────────────────────
template<typename T>
class Stack {
private:
    std::vector<T> data;
    int max_size;

public:
    Stack ( int maxSize = 100 ) : max_size(maxSize) {}

    void push ( const T& value ) {
        if (data.size() >= static_cast<size_t>(max_size)) {
            throw std::overflow_error("Stack overflow");
        }
        data.push_back(value);
    }

    T pop ( ) {
        if (data.empty()) {
            throw std::underflow_error("Stack underflow");
        }
        T top = data.back();
        data.pop_back();
        return top;
    }

    bool empty() const {return data.empty();}
    size_t size() const { return data.size() ; }
};

// ── CASE 2: Inheritance and virtual functions ─────────────────────────────
class Shape {
public:
    virtual double area() const = 0;
    virtual std::string name() const = 0;
    virtual ~Shape() = default;

    void print () const {
        std::cout << name() << ": " << area() << std::endl;
    }
};

class Circle : public Shape {
    double radius;
public:
    Circle ( double r ) : radius(r) {}
    double area() const override { return 3.14159 * radius * radius; }
    std::string name() const override { return "Circle"; }
};

class Rectangle : public Shape {
    double width,height;
public:
    Rectangle(double w,double h):width(w),height(h){}
    double area() const override{return width*height;}
    std::string name() const override{return "Rectangle";}
};

// ── CASE 3: Lambda and algorithms ─────────────────────────────────────────
void sort_and_print ( std::vector<int>& v ) {
    std::sort(v.begin(),v.end(),[](int a,int b){return a < b;});
    std::for_each(v.begin(),v.end(),[](int x){std::cout<<x<<" ";});
    std::cout << std::endl;
}

// ── CASE 4: Smart pointers ────────────────────────────────────────────────
std::unique_ptr<Shape> create_shape ( const std::string& type ) {
    if (type == "circle") {
        return std::make_unique<Circle>(5.0);
    } else if (type == "rectangle") {
        return std::make_unique<Rectangle>(4.0,6.0);
    }
    return nullptr;
}

// ── CASE 5: Long function signature ───────────────────────────────────────
template<typename InputIt, typename OutputIt, typename UnaryPredicate>
OutputIt copy_if_transform(InputIt first, InputIt last, OutputIt d_first, UnaryPredicate pred) {
    for (; first != last; ++first) {
        if (pred(*first)) {
            *d_first++ = *first;
        }
    }
    return d_first;
}

// ── CASE 6: Trailing whitespace ───────────────────────────────────────────
int main ( ) {   
    Stack<int> s;   
    s.push(1);   
    s.push(2);   
    std::cout << s.pop() << std::endl;   
    return 0;
}
