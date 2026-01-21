/// Parses Dart locator patterns using the `nom` parser combinator library.
///
/// This module defines a `Locator` enum to represent different types of locators
/// and provides functions to parse these locators from a given input string.
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::multispace0,
    multi::many0,
    sequence::{delimited, tuple},
};

use crate::localisation::is_alphanumeric_or_underscore;

#[derive(Debug, PartialEq)]
pub enum Locator {
    Register(String),
    Get(String),
    Import,
}

/// Parser to extract the class being registered and used the GetIt dart locator package using nom
fn register_locator(input: &str) -> IResult<&str, Locator> {
    let (rest, (_, _, _, class)) = tuple((
        multispace0,
        tag("register"),
        take_until("<"),
        delimited(tag("<"), is_alphanumeric_or_underscore, tag(">")),
    ))(input)?;
    Ok((rest, Locator::Register(class.to_string())))
}

fn find_locator(input: &str) -> IResult<&str, ()> {
    let (r, _) = tuple((take_until("locator."), tag("locator.")))(input)?;
    Ok((r, ()))
}

fn find_locator_alt(input: &str) -> IResult<&str, ()> {
    let (r, _) = tuple((take_until("locator<"), tag("locator<")))(input)?;
    Ok((r, ()))
}

fn get_locator(input: &str) -> IResult<&str, Locator> {
    let (s, (_, l)) = tuple((find_locator, alt((import, register_locator, get, get_alt))))(input)?;
    Ok((s, l))
}

fn get_locator_alt(input: &str) -> IResult<&str, Locator> {
    let (s, (_, l)) = tuple((
        find_locator_alt,
        alt((import, register_locator, get, get_alt)),
    ))(input)?;
    Ok((s, l))
}

fn import(input: &str) -> IResult<&str, Locator> {
    let (s, _) = tag("dart")(input)?;
    Ok((s, Locator::Import))
}

/// Parses multiple locator patterns from the input string
///
/// Patterns can be of the form:
/// - `locator.register...<GetIt>(() => ...);`
/// - `locator.get<GetIt>();`
/// - `locator<GetIt>();`
pub fn locator(input: &str) -> IResult<&str, Vec<Locator>> {
    let (r1, l) = many0(get_locator)(input)?;
    let (r2, x) = many0(get_locator_alt)(input)?;
    let mut s = l;
    s.extend(x);
    if r1.len() > r2.len() {
        Ok((r2, s))
    } else {
        Ok((r1, s))
    }
}

fn get(input: &str) -> IResult<&str, Locator> {
    let (remaining, (_, _, _, class)) = tuple((
        multispace0,
        tag("get"),
        take_until("<"),
        delimited(tag("<"), is_alphanumeric_or_underscore, tag(">")),
    ))(input)?;
    Ok((remaining, Locator::Get(class.to_string())))
}

fn get_alt(input: &str) -> IResult<&str, Locator> {
    let (remaining, (_, class)) = tuple((multispace0, take_until(">")))(input)?;
    Ok((remaining, Locator::Get(class.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locator() {
        let input = r#"register<GetIt>();"#;
        let result = register_locator(input);
        assert_eq!(result, Ok(("();", Locator::Register("GetIt".to_string()))));
    }

    #[test]
    fn test_locator_singleton() {
        let input = r#"registerLazySingleton<GetIt>();"#;
        let result = register_locator(input);
        assert_eq!(result, Ok(("();", Locator::Register("GetIt".to_string()))));
    }

    #[test]
    fn test_locator_factory() {
        let input = r#"registerFactory<GetIt>();"#;
        let result = register_locator(input);
        assert_eq!(result, Ok(("();", Locator::Register("GetIt".to_string()))));
    }

    #[test]
    fn test_locator_parent() {
        let input = r#"locator.register<GetIt>();"#;
        let result = get_locator(input);
        assert_eq!(result, Ok(("();", Locator::Register("GetIt".to_string()))));
    }

    #[test]
    fn test_locator_singleton_parent() {
        let input = r#"locator.registerLazySingleton<GetIt>();"#;
        let result = get_locator(input);
        assert_eq!(result, Ok(("();", Locator::Register("GetIt".to_string()))));
    }

    #[test]
    fn test_locator_factory_parent() {
        let input = r#"locator.registerFactory<GetIt>();"#;
        let result = get_locator(input);
        assert_eq!(result, Ok(("();", Locator::Register("GetIt".to_string()))));
    }

    #[test]
    fn test_get_locator() {
        let input = r#"locator.get<GetIt>();"#;
        let result = get_locator(input);
        assert_eq!(result, Ok(("();", Locator::Get("GetIt".to_string()))));
    }

    #[test]
    fn test_get_locator_alt() {
        let input = r#"locator<GetIt>();"#;
        let result = get_locator_alt(input);
        assert_eq!(result, Ok((">();", Locator::Get("GetIt".to_string()))));
    }

    #[test]
    fn test_get_locator_in_register() {
        let input = r#"  locator.registerFactory<CreditApplicationContractBloc>(
      () => CreditApplicationContractBloc(locator.get<DownloadContractUseCase>(),
       locator<DownloadContractsUseCase>()));"#;
        let result = locator(input);
        assert_eq!(
            result,
            Ok((
                ">()));",
                vec![
                    Locator::Register("CreditApplicationContractBloc".to_string()),
                    Locator::Get("DownloadContractUseCase".to_string()),
                    Locator::Get("DownloadContractsUseCase".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_locators_multiple() {
        let input = r#"
    locator.registerLazySingleton<ChatPageBloc>(() => ChatPageBloc(
      locator<UserInfoNotifier>(),
      locator<ChatConnectionNotifier>(),
      domain.createWebSocketConnectionUseCase,
      domain.startChatUseCase,
      domain.handleIncomingWebsocketUseCase,
      domain.sendMessageUseCase,
      domain.disconnectAllUseCase,
      domain.getAttachmentUseCase,
      domain.sendAttachmentUseCase,
      domain.createChatCacheUseCase,
      domain.updateChatCacheUseCase,
      appConfig.testMode));"#;

        let result = locator(input);
        assert_eq!(
            result,
            Ok((
                r#">(),
      domain.createWebSocketConnectionUseCase,
      domain.startChatUseCase,
      domain.handleIncomingWebsocketUseCase,
      domain.sendMessageUseCase,
      domain.disconnectAllUseCase,
      domain.getAttachmentUseCase,
      domain.sendAttachmentUseCase,
      domain.createChatCacheUseCase,
      domain.updateChatCacheUseCase,
      appConfig.testMode));"#,
                vec![
                    Locator::Register("ChatPageBloc".to_string()),
                    Locator::Get("UserInfoNotifier".to_string()),
                    Locator::Get("ChatConnectionNotifier".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_file_contents() {
        let input = r#"import 'package:olympus/core/core.dart';
import 'package:olympus/core/logger/app_logger.dart';
import 'package:olympus/core/notifiers/chat_connection_notifier.dart';
import 'package:olympus/core/notifiers/user_info_notifier.dart';
import 'package:olympus/domain/domain_service_operations.dart';
import 'package:olympus/presentation/app/app_config.dart';
import 'package:olympus/presentation/features/chat/bloc/chat_page_bloc.dart';
import 'package:olympus/presentation/features/chat_menu/bloc/chat_menu_bloc.dart';
import 'package:olympus/presentation/features/chat_menu_message_history/bloc/chat_menu_message_history_bloc.dart';
import 'package:olympus/presentation/features/chat_schedule_callback_slots_available/bloc/chat_schedule_callback_bloc.dart';
import 'package:olympus/presentation/features/chat_view_schedule_callback/bloc/view_schedule_callback_bloc.dart';
import 'package:olympus/presentation/features/faq_categories/bloc/faq_categories_page_bloc.dart';
import 'package:olympus/presentation/features/faq_drawer_search/bloc/faq_drawer_search_bloc.dart';
import 'package:olympus/presentation/features/faq_products/bloc/faq_products_page_bloc.dart';
import 'package:olympus/presentation/features/faq_search/bloc/faq_search_page_bloc.dart';
import 'package:olympus/presentation/features/faq_tell_me_more/bloc/faq_tell_me_more_bloc.dart';
import 'package:olympus/presentation/features/notifications/bloc/notification_page_bloc.dart';
import 'package:olympus/presentation/features/notifications_messages/bloc/notifications_messages_bloc.dart';
import 'package:olympus/presentation/features/order_tracking_status/bloc/order_tracking_status_bloc.dart';
import 'package:olympus/presentation/features/order_tracking_status_list/bloc/order_tracking_status_list_bloc.dart';
import 'package:olympus/presentation/util/locator.dart';

Future<void> serviceOperationsLocator(
    {required CoreSubModule core,
    required AppConfig appConfig,
    required ServiceOperationsDomainModule domain}) async {
  locator.registerLazySingleton<ChatConnectionNotifier>(
      () => ChatConnectionNotifier(domain.disconnectAllUseCase));

  // FAQ
  locator.registerFactory<FAQProductPageBloc>(
    () => FAQProductPageBloc(
        domain.getMostAskedFaqUseCase, domain.getMainFaqCategoriesUseCase, locator.get<AppLogger>()),
  );

  locator.registerFactory<FAQCategoryPageBloc>(
    () => FAQCategoryPageBloc(domain.getFaqCategoriesListUseCase),
  );
  locator.registerFactory<FAQSearchPageBloc>(
    () => FAQSearchPageBloc(
        domain.getFaqProductsUseCase, domain.getFaqCategoriesUseCase, domain.getFaqCategoriesListUseCase),
  );

  locator.registerFactory<FAQTellMeMorePageBloc>(
    () => FAQTellMeMorePageBloc(),
  );

  locator.registerFactory<FaqDrawerSearchBloc>(
    () => FaqDrawerSearchBloc(
        domain.getMostAskedFaqUseCase, domain.getFaqProductsUseCase, locator.get<AppLogger>()),
  );

  //Notifications
  locator.registerFactory<NotificationsPageBloc>(() => NotificationsPageBloc(
        domain.getNotificationsUseCase,
        domain.notificationStatusUpdateUseCase,
        locator.get<UserInfoNotifier>(),
      ));

  locator.registerFactory<NotificationsMessagesBloc>(() => NotificationsMessagesBloc(
        domain.notificationStatusUpdateUseCase,
        locator.get<UserInfoNotifier>(),
      ));

  //Chat + Schedule callback
  locator.registerLazySingleton<ChatPageBloc>(() => ChatPageBloc(
      locator<UserInfoNotifier>(),
      locator<ChatConnectionNotifier>(),
      domain.createWebSocketConnectionUseCase,
      domain.startChatUseCase,
      domain.handleIncomingWebsocketUseCase,
      domain.sendMessageUseCase,
      domain.disconnectAllUseCase,
      domain.getAttachmentUseCase,
      domain.sendAttachmentUseCase,
      domain.createChatCacheUseCase,
      domain.updateChatCacheUseCase,
      appConfig.testMode));

  locator.registerFactory<ChatMenuBloc>(() => ChatMenuBloc(
        createChatCacheUseCase: domain.createChatCacheUseCase,
        readChatCacheUseCase: domain.readChatCacheUseCase,
        readAllChatCacheUseCase: domain.readAllChatCacheUseCase,
        updateChatCacheUseCase: domain.updateChatCacheUseCase,
        deleteChatCacheUseCase: domain.deleteChatCacheUseCase,
        disconnectAllUseCase: domain.disconnectAllUseCase,
        chatConnectionNotifier: locator<ChatConnectionNotifier>(),
      ));

  locator.registerFactory<ChatScheduleCallbackBloc>(() => ChatScheduleCallbackBloc(
      domain.scheduleCallbackUseCase,
      domain.getSlotsUseCase,
      domain.deleteScheduledCallbackUseCase,
      domain.updateScheduledCallbackUseCase,
      domain.getScheduledCallbackUseCase));

  locator.registerFactory<ViewScheduleCallbackBloc>(() => ViewScheduleCallbackBloc(
      domain.getSlotsUseCase,
      domain.deleteScheduledCallbackUseCase,
      domain.updateScheduledCallbackUseCase,
      domain.scheduleCallbackUseCase,
      locator<UserInfoNotifier>()));

  locator.registerFactory<ChatMenuMessageHistoryBloc>(() => ChatMenuMessageHistoryBloc(
        disconnectAllUseCase: domain.disconnectAllUseCase,
        chatConnectionNotifier: locator<ChatConnectionNotifier>(),
      ));

  //Order tracking
  locator.registerFactory<OrderTrackingStatusListBloc>(() => OrderTrackingStatusListBloc(
        searchOrdersUseCase: domain.searchOrdersUseCase,
      ));
  locator.registerFactory<OrderTrackingStatusBloc>(() => OrderTrackingStatusBloc(
        getOrderUseCase: domain.getOrderUseCase,
      ));
}
"#;

        let result = locator(input);

        assert_eq!(
            result,
            Ok((
                "(() => OrderTrackingStatusBloc(\n        getOrderUseCase: domain.getOrderUseCase,\n      ));\n}\n",
                vec![
                    Locator::Import,
                    Locator::Register("ChatConnectionNotifier".to_string()),
                    Locator::Register("FAQProductPageBloc".to_string()),
                    Locator::Get("AppLogger".to_string()),
                    Locator::Register("FAQCategoryPageBloc".to_string()),
                    Locator::Register("FAQSearchPageBloc".to_string()),
                    Locator::Register("FAQTellMeMorePageBloc".to_string()),
                    Locator::Register("FaqDrawerSearchBloc".to_string()),
                    Locator::Get("AppLogger".to_string()),
                    Locator::Register("NotificationsPageBloc".to_string()),
                    Locator::Get("UserInfoNotifier".to_string()),
                    Locator::Register("NotificationsMessagesBloc".to_string()),
                    Locator::Get("UserInfoNotifier".to_string()),
                    Locator::Register("ChatPageBloc".to_string()),
                    Locator::Register("ChatMenuBloc".to_string()),
                    Locator::Register("ChatScheduleCallbackBloc".to_string()),
                    Locator::Register("ViewScheduleCallbackBloc".to_string()),
                    Locator::Register("ChatMenuMessageHistoryBloc".to_string()),
                    Locator::Register("OrderTrackingStatusListBloc".to_string()),
                    Locator::Register("OrderTrackingStatusBloc".to_string()),
                    Locator::Get("UserInfoNotifier".to_string()),
                    Locator::Get("ChatConnectionNotifier".to_string()),
                    Locator::Get("ChatConnectionNotifier".to_string()),
                    Locator::Get("UserInfoNotifier".to_string()),
                    Locator::Get("ChatConnectionNotifier".to_string()),
                ]
            ))
        );
    }
}
