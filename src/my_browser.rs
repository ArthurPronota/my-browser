// Управление браузером

use serde_json::json;

use std::{
        error::Error, time::Duration, 
    };

use playhard::{
        launcher::LaunchOptions,
        Browser,
        //Result,
    };

use playhard_automation::{
        AutomationError, LoadState, Locator, NavigateOptions, Page
    } ;

use playhard_cdp::PageNavigateResult ;

use log::{  // импорт макросов логирования
        LevelFilter::Info, debug, error, info, trace, warn
    } ;

/// Управляющая структура браузера
pub struct MyBrowser {
    options:    Option<LaunchOptions>,  // опции запуска браузера
    browser:    Option<Browser>,        // браузер
    pub tab:    Option<Page>,           // вкладка (в ней открываются страницы)
}

const INIT_PART: &str = "[INIT]" ;  // наименование раздела инициализации

const SHUTDOWN_PART: &str = "[SHUTDOWN]" ;

const TIMEOUT_PAGE_LOAD: u64 = 120 ;    // время ожидания загрузки  страницы (secs)

const LOAD_URL_PART: &str = "[LOAD_URL]" ;

// реализация методов для браузера
impl MyBrowser {

    /// инициация структуры браузера
    pub fn new() ->MyBrowser {
        trace!("{INIT_PART} Начальная инициализация MyBrowser") ;
        MyBrowser { 
            options: None, 
            browser: None, 
            tab: None 
        }
    }

    /// инициация структуры браузера
    pub fn new2(&mut self) {
        trace!("{INIT_PART} Начальная инициализация MyBrowser (2)") ;
        self.options = None ;
        self.browser = None ;
        self.tab = None ;
    }

    /// установка опций для старта браузера
    pub fn set_options(
            &mut self,
            options: &LaunchOptions
           )
    {
        self.options = Some(options.clone()) ;
    }

    /// Запуск браузера (инициализация self.browser)
    pub async fn launch_browser(&mut self) -> core::result::Result<(), Box<dyn Error>> {
        info!("{INIT_PART} Перед запуском браузера") ;
        match &self.options {
            Some(l_opt) => {
                self.browser = match Browser::launch(l_opt.clone()).await {
                    Ok(br) => {
                        Some(br)
                    },
                    Err(err) => {
                        error!("{INIT_PART} Ошибка инициализации браузера: {err}") ;
                        return Err(" Ошибка инициализации браузера: {err}".into());
                    },
                } ;
            },
            None => {
                error!("{INIT_PART} Опции браузера не установлены") ;
                return Err("Опции браузера не установлены".into());
            }
        }

        info!("{INIT_PART} Браузер запущен") ;
        Ok(())
    }

    // признак необходимости инициализации
    pub fn need_run(&self) ->bool {
        self.browser.is_none()
    }

    /// Закрытие браузера
    pub async fn shutdown_browser(&mut self) ->Result<(), Box<dyn Error>> {
        if let Some(brow) = self.browser.take() {   // берёт значение из Option, оставляя на его месте None
            if let Err(err) = brow.shutdown().await {
                let text_mess = format!("Ошибка закрытия браузера: {err}") ;
                error!("{SHUTDOWN_PART} {text_mess}") ;
                self.tab = None ;
                return Err(text_mess.into());
            }
        } else {
            let text_mess = "Браузер не инициализирован";
            error!("{SHUTDOWN_PART} {text_mess}") ;
            return Err(text_mess.into()) ;
        }

        Ok(())
    }

    /// открытие новой вкладки
    pub async fn new_tab(&mut self) -> core::result::Result<(), Box<dyn Error>> {

        match &self.browser {
            Some(br) => {
                if let Some(pg) = &self.tab {
                    let text_mess = "Вкладка уже инициализирована" ;
                    error!("{INIT_PART} {text_mess}") ;
                    return Err(text_mess.into());
                }

                self.tab = match br.new_page().await {
                    Ok(pg) => {
                        Some(pg)
                    },
                    Err(err) => {
                        let text_mess = format!("Ошибка открытия новой вкладки: {err}") ;
                        error!("{INIT_PART} {text_mess}") ;
                        return Err(text_mess.into());
                    },
                } ;
            },
            None => {
                let text_mess = "Браузер не инициализирован" ;
                error!("{INIT_PART} {text_mess}") ;
                return Err(text_mess.into());
            }
        }

        Ok(())
    }

    /// признак необходимости открытия new tab
    pub fn need_new_tab(&self) ->bool {
        self.tab.is_none()
    }

    /// Загрузка url в Tab
    pub async fn load_url(&mut self, url: &str) ->Result<PageNavigateResult, Box<dyn Error>> {
        if url.trim().is_empty() {
            let text_mess = "url is empty" ;
            error!("{LOAD_URL_PART} {text_mess}") ;
            return Err(format!("{LOAD_URL_PART} {text_mess}").into());
        }

        match &self.tab {
            Some(tb) => {
                //tb.goto_with_options(url, options)
                match 
                    //tb.goto(url).await    // 20260517
                    tb.goto_with_options(   // 20260517
                        url, 
                        NavigateOptions{
                            wait_until: LoadState::DomContentLoaded, 
                            timeout: Duration::from_secs(TIMEOUT_PAGE_LOAD)
                        }).await
                {
                    Ok(page_nav) => Ok(page_nav),
                    Err(err) => {
                        let text_mess = format!("Ошибка загрузки страницы: {url}, error: {err}") ;
                        error!("{LOAD_URL_PART} {text_mess}") ;
                        return Err(text_mess.into());
                    },
                }
            },
            None => {
                let text_mess = format!("self.tab не инициализирован") ;
                error!("{LOAD_URL_PART} {text_mess}") ;
                return Err(text_mess.into());
            }
        }
    }

    /// очиститьвсе cookie
    pub async fn clear_all_cooie(&mut self) ->Result<(), Box<dyn  Error>> {
        let name_part = "[CLEAR_ALL_COOKIE]" ;
        info!("{name_part} Before") ;
        match &self.browser {
            Some(brw) => {
                match brw
                        .cdp()
                        .call_raw("Storage.clearCookies", json!({}))
                        .await {
                    Ok(_) => {
                            info!("{name_part} У браузера очишены cookie") ;
                            Ok(())
                    },
                    Err(err) => {
                        let text_mess = format!("{name_part} Ошибка очисткии cookie браузера: {err}") ;
                        error!("{text_mess}") ;
                        Err(text_mess.into())
                    }
                }
             },
            None => {
                let text_mess = format!("{name_part} Браузер не инициализирован") ;
                error!("{text_mess}") ;
                Err(text_mess.into())
            }
        }
    }
}
