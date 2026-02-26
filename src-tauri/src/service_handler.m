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

  // 用 NSTask 直接执行可执行文件（绕过 Launch Services），保证无论 app
  // 是否已在运行都能创建新进程并传递 --compress 参数：
  // - app 未运行：新进程成为首个实例，直接进入后台压缩模式后退出
  // - app 已运行：单实例插件将参数转发给已有进程后新进程退出
  NSString *exePath = [[NSBundle mainBundle] executablePath];
  NSMutableArray<NSString *> *args =
      [NSMutableArray arrayWithObject:@"--compress"];
  [args addObjectsFromArray:paths];

  NSTask *task = [[NSTask alloc] init];
  task.executableURL = [NSURL fileURLWithPath:exePath];
  task.arguments = args;
  [task launchAndReturnError:nil];
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
