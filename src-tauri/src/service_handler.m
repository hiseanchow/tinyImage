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

  // 直接通过 NSWorkspace 启动 app 并传递 --compress 参数：
  // 不依赖 URL scheme 路由，避免未签名 app 无法通过 tinyimage:// 被启动的问题。
  // 若 app 已在运行，单实例插件会将参数转发给已有进程后自动退出。
  NSURL *appURL = [[NSBundle mainBundle] bundleURL];
  NSMutableArray<NSString *> *args =
      [NSMutableArray arrayWithObject:@"--compress"];
  [args addObjectsFromArray:paths];

  NSWorkspaceOpenConfiguration *config =
      [NSWorkspaceOpenConfiguration configuration];
  config.arguments = args;
  config.activates = NO; // 后台启动，不抢占前台焦点

  [[NSWorkspace sharedWorkspace] openApplicationAtURL:appURL
                                        configuration:config
                                    completionHandler:nil];
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
