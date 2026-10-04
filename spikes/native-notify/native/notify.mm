#import <AppKit/AppKit.h>
#import <UserNotifications/UserNotifications.h>
#include <napi.h>
#include <functional>

// Main-only spike addon. All Cocoa callbacks cross into JS through a TSFN.
static Napi::ThreadSafeFunction sink;
static void Emit(NSDictionary *value) {
  NSData *data = [NSJSONSerialization dataWithJSONObject:value options:0 error:nil];
  auto text = new std::string((const char *)data.bytes, data.length);
  if (sink.NonBlockingCall(text, [](Napi::Env env, Napi::Function callback, std::string *text) {
    callback.Call({Napi::String::New(env, *text)});
    delete text;
  }) != napi_ok) delete text;
}
static BOOL Owned(NSString *identifier) {
  return [identifier hasPrefix:@"rem:"] || [identifier hasPrefix:@"one:"];
}
static NSArray *Rows(NSArray<UNNotificationRequest *> *requests) {
  NSMutableArray *rows = [NSMutableArray array];
  for (UNNotificationRequest *r in requests) {
    [rows addObject:@{@"id":r.identifier, @"title":r.content.title, @"body":r.content.body,
      @"hash":r.content.userInfo[@"hash"] ?: @"", @"repeats":@(r.trigger.repeats),
      @"trigger":NSStringFromClass(r.trigger.class)}];
  }
  return rows;
}
static NSString *Auth(UNAuthorizationStatus status) {
  switch (status) {
    case UNAuthorizationStatusAuthorized: return @"authorized";
    case UNAuthorizationStatusDenied: return @"denied";
    case UNAuthorizationStatusNotDetermined: return @"notDetermined";
    case UNAuthorizationStatusProvisional: return @"provisional";
    default: return @"unknown";
  }
}

@interface NotifyDelegate : NSObject <UNUserNotificationCenterDelegate>
@property NSMutableDictionary<NSString *, id> *completions;
@end
@implementation NotifyDelegate
- (instancetype)init {
  self = [super init];
  if (self) {
    self.completions = [NSMutableDictionary dictionary];
    __weak NotifyDelegate *weakSelf = self;
    [[[NSWorkspace sharedWorkspace] notificationCenter] addObserverForName:NSWorkspaceDidWakeNotification object:nil queue:nil usingBlock:^(NSNotification *n) {
      if (weakSelf) Emit(@{@"event":@"refresh", @"reason":@"wake"});
    }];
    [[NSNotificationCenter defaultCenter] addObserverForName:NSSystemTimeZoneDidChangeNotification object:nil queue:nil usingBlock:^(NSNotification *n) {
      if (weakSelf) Emit(@{@"event":@"refresh", @"reason":@"timezone"});
    }];
  }
  return self;
}
- (void)userNotificationCenter:(UNUserNotificationCenter *)center willPresentNotification:(UNNotification *)notification withCompletionHandler:(void (^)(UNNotificationPresentationOptions))completion {
  Emit(@{@"event":@"refresh", @"reason":@"delivered", @"id":notification.request.identifier});
  completion(UNNotificationPresentationOptionBanner | UNNotificationPresentationOptionList | UNNotificationPresentationOptionSound);
}
- (void)userNotificationCenter:(UNUserNotificationCenter *)center didReceiveNotificationResponse:(UNNotificationResponse *)response withCompletionHandler:(void (^)(void))completion {
  NSString *token = NSUUID.UUID.UUIDString;
  @synchronized (self) { self.completions[token] = [completion copy]; }
  NSDictionary *info = response.notification.request.content.userInfo;
  Emit(@{@"event":@"action", @"token":token, @"action":response.actionIdentifier,
    @"id":response.notification.request.identifier, @"memoryId":info[@"memoryId"] ?: @"",
    @"occurrenceAt":info[@"occurrenceAt"] ?: @"", @"receivedAtMs":@([NSDate.date timeIntervalSince1970] * 1000)});
}
@end
static NotifyDelegate *delegate;
static void Reply(NSString *rid, id value, NSString *error) {
  Emit(error ? @{@"rid":rid, @"error":error} : @{@"rid":rid, @"value":value ?: @YES});
}
static UNNotificationRequest *Request(NSDictionary *item) {
  UNMutableNotificationContent *content = [UNMutableNotificationContent new];
  content.title = item[@"title"]; content.body = item[@"body"]; content.categoryIdentifier = item[@"category"];
  content.sound = UNNotificationSound.defaultSound;
  content.userInfo = @{@"memoryId":item[@"memoryId"], @"hash":item[@"hash"], @"occurrenceAt":item[@"occurrenceAt"] ?: item[@"fireAt"]};
  UNNotificationTrigger *trigger;
  NSDictionary *repeat = item[@"repeat"];
  if ([repeat isKindOfClass:NSDictionary.class]) {
    NSDateComponents *c = [NSDateComponents new];
    c.calendar = [[NSCalendar alloc] initWithCalendarIdentifier:NSCalendarIdentifierGregorian];
    c.timeZone = [NSTimeZone timeZoneWithName:repeat[@"tz"]];
    for (NSString *key in @[@"month", @"day", @"weekday", @"hour", @"minute", @"second"])
      if (repeat[key]) [c setValue:repeat[key] forKey:key];
    trigger = [UNCalendarNotificationTrigger triggerWithDateMatchingComponents:c repeats:YES];
  } else {
    NSISO8601DateFormatter *f = [NSISO8601DateFormatter new];
    f.formatOptions = NSISO8601DateFormatWithInternetDateTime | NSISO8601DateFormatWithFractionalSeconds;
    NSDate *fire = [f dateFromString:item[@"fireAt"]];
    trigger = [UNTimeIntervalNotificationTrigger triggerWithTimeInterval:MAX(0.1, fire.timeIntervalSinceNow) repeats:NO];
  }
  return [UNNotificationRequest requestWithIdentifier:item[@"id"] content:content trigger:trigger];
}
static void AddItems(NSArray *items, NSUInteger index, NSMutableArray *added, NSString *rid, NSArray *stale, NSUInteger unchanged) {
  UNUserNotificationCenter *center = UNUserNotificationCenter.currentNotificationCenter;
  if (index == items.count) {
    [center removePendingNotificationRequestsWithIdentifiers:stale];
    Reply(rid, @{@"added":added, @"removed":stale, @"unchanged":@(unchanged)}, nil);
    return;
  }
  NSDictionary *item = items[index];
  [center addNotificationRequest:Request(item) withCompletionHandler:^(NSError *error) {
    if (error) { Reply(rid, nil, error.localizedDescription); return; }
    [added addObject:item[@"id"]];
    AddItems(items, index + 1, added, rid, stale, unchanged);
  }];
}
static void Send(const Napi::CallbackInfo &args) {
  NSString *rid = [NSString stringWithUTF8String:args[0].As<Napi::String>().Utf8Value().c_str()];
  NSString *method = [NSString stringWithUTF8String:args[1].As<Napi::String>().Utf8Value().c_str()];
  std::string raw = args[2].As<Napi::String>().Utf8Value();
  id input = [NSJSONSerialization JSONObjectWithData:[NSData dataWithBytes:raw.data() length:raw.size()] options:NSJSONReadingFragmentsAllowed error:nil];
  UNUserNotificationCenter *center = UNUserNotificationCenter.currentNotificationCenter;
  if ([method isEqual:@"requestAuthorization"]) {
    [center requestAuthorizationWithOptions:UNAuthorizationOptionAlert | UNAuthorizationOptionSound | UNAuthorizationOptionBadge completionHandler:^(BOOL granted, NSError *error) {
      Reply(rid, @(granted), error.localizedDescription);
    }];
  } else if ([method isEqual:@"status"]) {
    [center getNotificationSettingsWithCompletionHandler:^(UNNotificationSettings *settings) {
      [center getPendingNotificationRequestsWithCompletionHandler:^(NSArray *requests) {
        Reply(rid, @{@"authorization":Auth(settings.authorizationStatus), @"pendingCount":@(requests.count),
          @"bundleId":NSBundle.mainBundle.bundleIdentifier ?: @"", @"bundleName":[NSBundle.mainBundle objectForInfoDictionaryKey:@"CFBundleDisplayName"] ?: @""}, nil);
      }];
    }];
  } else if ([method isEqual:@"pending"]) {
    [center getPendingNotificationRequestsWithCompletionHandler:^(NSArray *requests) { Reply(rid, Rows(requests), nil); }];
  } else if ([method isEqual:@"delivered"]) {
    [center getDeliveredNotificationsWithCompletionHandler:^(NSArray<UNNotification *> *notifications) {
      NSMutableArray *requests = [NSMutableArray array];
      for (UNNotification *n in notifications) [requests addObject:n.request];
      Reply(rid, Rows(requests), nil);
    }];
  } else if ([method isEqual:@"removeAll"]) {
    [center getPendingNotificationRequestsWithCompletionHandler:^(NSArray *requests) {
      NSMutableArray *ids = [NSMutableArray array];
      for (UNNotificationRequest *r in requests) if (Owned(r.identifier) && [r.identifier hasPrefix:input]) [ids addObject:r.identifier];
      [center removePendingNotificationRequestsWithIdentifiers:ids];
      [center getDeliveredNotificationsWithCompletionHandler:^(NSArray *notifications) {
        NSMutableArray *delivered = [NSMutableArray array];
        for (UNNotification *n in notifications) if (Owned(n.request.identifier) && [n.request.identifier hasPrefix:input]) [delivered addObject:n.request.identifier];
        [center removeDeliveredNotificationsWithIdentifiers:delivered];
        Reply(rid, ids, nil);
      }];
    }];
  } else if ([method isEqual:@"apply"]) {
    [center getNotificationSettingsWithCompletionHandler:^(UNNotificationSettings *settings) {
      if (settings.authorizationStatus != UNAuthorizationStatusAuthorized && settings.authorizationStatus != UNAuthorizationStatusProvisional) { Reply(rid, nil, @"permission denied; existing requests preserved"); return; }
      [center getPendingNotificationRequestsWithCompletionHandler:^(NSArray *requests) {
        NSMutableDictionary *hashes = [NSMutableDictionary dictionary];
        for (UNNotificationRequest *r in requests) hashes[r.identifier] = r.content.userInfo[@"hash"] ?: @"";
        NSMutableSet *desired = [NSMutableSet set];
        NSMutableArray *changed = [NSMutableArray array], *stale = [NSMutableArray array];
        NSUInteger unchanged = 0;
        for (NSDictionary *item in input) {
          [desired addObject:item[@"id"]];
          if ([hashes[item[@"id"]] isEqual:item[@"hash"]]) unchanged++; else [changed addObject:item];
        }
        for (UNNotificationRequest *r in requests) if (Owned(r.identifier) && ![desired containsObject:r.identifier]) [stale addObject:r.identifier];
        AddItems(changed, 0, [NSMutableArray array], rid, stale, unchanged);
      }];
    }];
  } else if ([method isEqual:@"finishAction"]) {
    void (^completion)(void);
    @synchronized (delegate) { completion = delegate.completions[input]; [delegate.completions removeObjectForKey:input]; }
    if (completion) completion();
    Reply(rid, @YES, nil);
  } else if ([method isEqual:@"testHooks"]) {
    [[[NSWorkspace sharedWorkspace] notificationCenter] postNotificationName:NSWorkspaceDidWakeNotification object:nil];
    [[NSNotificationCenter defaultCenter] postNotificationName:NSSystemTimeZoneDidChangeNotification object:nil];
    Reply(rid, @YES, nil);
  } else Reply(rid, nil, @"unknown command");
}
static Napi::Object Init(Napi::Env env, Napi::Object exports) {
  exports.Set("start", Napi::Function::New(env, [](const Napi::CallbackInfo &args) {
    sink = Napi::ThreadSafeFunction::New(args.Env(), args[0].As<Napi::Function>(), "notify", 0, 1);
    sink.Unref(args.Env());
    delegate = [NotifyDelegate new];
    UNUserNotificationCenter *center = UNUserNotificationCenter.currentNotificationCenter;
    center.delegate = delegate; // Before app.whenReady / didFinishLaunching.
    UNNotificationAction *done = [UNNotificationAction actionWithIdentifier:@"DONE" title:@"Done" options:0];
    UNNotificationAction *snooze = [UNNotificationAction actionWithIdentifier:@"SNOOZE" title:@"Snooze" options:0];
    UNNotificationAction *open = [UNNotificationAction actionWithIdentifier:@"OPEN" title:@"Open" options:UNNotificationActionOptionForeground];
    UNNotificationAction *focus = [UNNotificationAction actionWithIdentifier:@"START_FOCUS" title:@"Start focus" options:UNNotificationActionOptionForeground];
    [center setNotificationCategories:[NSSet setWithArray:@[
      [UNNotificationCategory categoryWithIdentifier:@"reminder" actions:@[done,snooze,open] intentIdentifiers:@[] options:0],
      [UNNotificationCategory categoryWithIdentifier:@"reminder-focus" actions:@[done,snooze,focus,open] intentIdentifiers:@[] options:0],
      [UNNotificationCategory categoryWithIdentifier:@"focus-phase" actions:@[] intentIdentifiers:@[] options:0]]]];
  }));
  exports.Set("send", Napi::Function::New(env, Send));
  return exports;
}
NODE_API_MODULE(notify, Init)
