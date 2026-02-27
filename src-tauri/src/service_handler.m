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

  // 通过 openURL 将压缩请求路由给 TinyImage：
  // - app 已运行：macOS 将 URL 投递给正在运行的进程，deep-link 处理器接收后在后台压缩
  // - app 未运行：macOS 启动 app（需 adhoc 签名使 URL scheme 在 Launch Services 中正确注册），
  //   app 启动后 deep-link 处理器接收 URL，以后台模式压缩完成后自动退出
  NSMutableCharacterSet *allowed =
      [[NSCharacterSet URLQueryAllowedCharacterSet] mutableCopy];
  [allowed removeCharactersInString:@"&=?#+%"];

  NSMutableString *urlStr = [@"tinyimage://compress?background=1" mutableCopy];
  for (NSString *path in paths) {
    NSString *encoded =
        [path stringByAddingPercentEncodingWithAllowedCharacters:allowed];
    if (!encoded)
      continue;
    [urlStr appendFormat:@"&file=%@", encoded];
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

// 切换为后台 app（隐藏 Dock 图标），供后台压缩模式调用
void setActivationPolicyAccessory(void) {
  dispatch_async(dispatch_get_main_queue(), ^{
    [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
  });
}
