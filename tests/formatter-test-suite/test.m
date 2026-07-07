#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>

// ── CASE 1: Interface declaration — spacing ───────────────────────────────
@interface Person : NSObject

@property (nonatomic, strong) NSString   *name;
@property (nonatomic, assign) NSInteger   age;
@property (nonatomic, strong)   NSArray<NSString *> *hobbies;

- (instancetype)initWithName:(NSString *)name age:(NSInteger)age;
- (NSString *)greet;
- (void)addHobby:(NSString *)hobby;

@end

// ── CASE 2: Implementation — indentation ─────────────────────────────────
@implementation Person

- (instancetype) initWithName:(NSString *)name age:(NSInteger)age {
    self = [super init];
    if (self) {
        _name = name;
        _age  = age;
        _hobbies = [NSMutableArray array];
    }
    return self;
}

- (NSString *) greet {
    return [NSString stringWithFormat:@"Hello, I'm %@ and I'm %ld years old.",
            _name, (long)_age];
}

- (void)addHobby:(NSString *)hobby {
    [(NSMutableArray *)_hobbies addObject:hobby];
}

@end

// ── CASE 3: Categories ────────────────────────────────────────────────────
@interface NSString (Utilities)

- (BOOL)isValidEmail;
- (NSString *)trimmedString;

@end

@implementation NSString (Utilities)

- (BOOL)isValidEmail {
    NSString *regex = @"[A-Z0-9a-z._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}";
    NSPredicate *test = [NSPredicate predicateWithFormat:@"SELF MATCHES %@", regex];
    return [test evaluateWithObject:self];
}

- (NSString *)trimmedString {
    return [self stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
}

@end

// ── CASE 4: Block syntax ──────────────────────────────────────────────────
void processItems(NSArray *items) {
    [items enumerateObjectsUsingBlock:^(id obj, NSUInteger idx, BOOL *stop) {
        NSLog(@"Item %lu: %@", (unsigned long)idx, obj);
        if (idx > 10) {
            *stop = YES;
        }
    }];
}

// ── CASE 5: Trailing whitespace ───────────────────────────────────────────
int main(int argc, const char *argv[]) {   
    @autoreleasepool {   
        Person *p = [[Person alloc] initWithName:@"Alice" age:30];   
        NSLog(@"%@", [p greet]);   
    }
    return 0;
}
