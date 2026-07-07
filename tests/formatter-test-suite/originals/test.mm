// ── CASE 1: Objective-C++ — mixing ObjC and C++ ──────────────────────────
#import <Foundation/Foundation.h>
#include <vector>
#include <string>
#include <algorithm>

// ── CASE 2: C++ class used from ObjC++ ────────────────────────────────────
class DataProcessor {
    std::vector<std::string> items;
    int max_size;
public:
    DataProcessor ( int maxSize = 100 ) : max_size(maxSize) {}

    void addItem ( const std::string& item ) {
        if ((int)items.size() < max_size) {
            items.push_back(item);
        }
    }

    std::vector<std::string> getSorted () const {
        auto sorted = items;
        std::sort(sorted.begin(),sorted.end());
        return sorted;
    }

    size_t count() const { return items.size() ; }
};

// ── CASE 3: ObjC interface wrapping C++ ────────────────────────────────────
@interface ItemManager : NSObject {
    DataProcessor *_processor;
}

- (instancetype)initWithMaxSize:(NSInteger)maxSize;
- (void)addItem:(NSString *)item;
- (NSArray<NSString *> *)sortedItems;
- (NSUInteger)count;

@end

// ── CASE 4: ObjC implementation ────────────────────────────────────────────
@implementation ItemManager

- (instancetype) initWithMaxSize:(NSInteger)maxSize {
    self = [super init];
    if (self) {
        _processor = new DataProcessor((int)maxSize);
    }
    return self;
}

- (void)addItem:(NSString *)item {
    _processor->addItem(std::string([item UTF8String]));
}

- (NSArray<NSString *> *)sortedItems {
    auto sorted = _processor->getSorted();
    NSMutableArray *result = [NSMutableArray array];
    for (const auto& s : sorted) {
        [result addObject:[NSString stringWithUTF8String:s.c_str()]];
    }
    return [result copy];
}

- (NSUInteger)count {
    return _processor->count();
}

- (void)dealloc {
    delete _processor;
}

@end

// ── CASE 5: Trailing whitespace ────────────────────────────────────────────
int main(int argc, const char *argv[]) {   
    @autoreleasepool {   
        ItemManager *mgr = [[ItemManager alloc] initWithMaxSize:10];   
        [mgr addItem:@"banana"];   
        [mgr addItem:@"apple"];   
        NSLog(@"%@", [mgr sortedItems]);   
    }
    return 0;
}
