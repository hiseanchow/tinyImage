#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>

// 读取 contextMenuEnabled 设置项（用于支持 Settings 面板的开关）
static BOOL isContextMenuEnabled(void) {
  NSString *appSupport =
      NSSearchPathForDirectoriesInDomains(NSApplicationSupportDirectory,
                                          NSUserDomainMask, YES)
          .firstObject;
  NSString *settingsPath =
      [appSupport stringByAppendingPathComponent:@"TinyImage/settings.json"];
  NSData *data = [NSData dataWithContentsOfFile:settingsPath];
  if (!data)
    return YES;
  NSDictionary *settings = [NSJSONSerialization JSONObjectWithData:data
                                                           options:0
                                                             error:nil];
  id value = settings[@"contextMenuEnabled"];
  if ([value isKindOfClass:[NSNumber class]]) {
    return [(NSNumber *)value boolValue];
  }
  return YES;
}

@interface TinyImageServiceHandler : NSObject
// NSMessage = "compressImages" → 方法名为 compressImages:userData:error:
- (void)compressImages:(NSPasteboard *)pboard
              userData:(NSString *)userData
                 error:(NSString **)error;
@end

@implementation TinyImageServiceHandler

- (void)compressImages:(NSPasteboard *)pboard
              userData:(NSString *)userData
                 error:(NSString **)error {
  if (!isContextMenuEnabled())
    return;

  // 优先读取文件 URL 列表（macOS 10.14+）
  NSArray<NSURL *> *urls = [pboard
      readObjectsForClasses:@[ [NSURL class] ]
                    options:@{NSPasteboardURLReadingFileURLsOnlyKey : @YES}];
  NSMutableArray<NSString *> *paths = [NSMutableArray array];
  for (NSURL *url in urls) {
    if (url.isFileURL) {
      [paths addObject:url.path];
    }
  }

  if (paths.count == 0)
    return;

  // 编码路径为 URL 参数值：需要对 & ? = # 等字符编码，但保留 /
  NSMutableCharacterSet *allowed =
      [[NSCharacterSet URLQueryAllowedCharacterSet] mutableCopy];
  [allowed removeCharactersInString:@"&=?#+%"];

  NSMutableString *urlStr = [@"tinyimage://compress?background=1" mutableCopy];
  BOOL first = NO;
  for (NSString *path in paths) {
    NSString *encoded =
        [path stringByAddingPercentEncodingWithAllowedCharacters:allowed];
    if (!encoded)
      continue;
    [urlStr appendFormat:@"%@file=%@", first ? @"?" : @"&", encoded];
    first = NO;
  }

  NSURL *url = [NSURL URLWithString:urlStr];
  if (url) {
    [[NSWorkspace sharedWorkspace] openURL:url];
  }
}

@end

// 全局 handler 实例（ARC 管理，防止被释放）
static TinyImageServiceHandler *gServiceHandler = nil;

void registerTinyImageService(void) {
  dispatch_async(dispatch_get_main_queue(), ^{
    gServiceHandler = [[TinyImageServiceHandler alloc] init];
    [NSApp setServicesProvider:gServiceHandler];
    NSUpdateDynamicServices();
  });
}
