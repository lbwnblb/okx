#[cfg(test)]
mod order_test{
    use reqwest::Client;
    use tokio_tungstenite::tungstenite::http::header;
    use crate::common::utils::{sign, HttpClientSimulation};

    #[tokio::test]
    pub async fn test_order(){
        let result = HttpClientSimulation::get("/api/v5/account/balance", Some(&vec![("ccy", "BTC")])).await;
        println!("{:?}", result.unwrap().text().await);
    }

    #[tokio::test]
    pub async fn okx_test_request(){

        // fn main() -> Result<(), Box<dyn std::error::Error>> {
        //     // let mut headers = header::HeaderMap::new();
        //     // headers.insert("accept", "application/json".parse().unwrap());
        //     // headers.insert("accept-language", "zh-CN,zh;q=0.9,en-GB;q=0.8,en-US;q=0.7,en;q=0.6".parse().unwrap());
        //     // headers.insert("content-type", "application/json".parse().unwrap());
        //     // headers.insert("ok-access-key", "f5bf9bf4-9889-465a-a4dc-c07ff2a0afff".parse().unwrap());
        //     // headers.insert("ok-access-passphrase", "123456".parse().unwrap());
        //     // headers.insert("ok-access-sign", "tIIkjv3TcKjmlwYrdV3YipZFGVN+M8vJYZVI3Re2hqQ=".parse().unwrap());
        //     // headers.insert("ok-access-timestamp", "2025-11-28T14:49:18.759Z".parse().unwrap());
        //     // headers.insert("priority", "u=1, i".parse().unwrap());
        //     // headers.insert("referer", "https://www.okx.com/zh-hans/demo-trading-explorer/v5/zh".parse().unwrap());
        //     // headers.insert("sec-ch-ua", "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"".parse().unwrap());
        //     // headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
        //     // headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
        //     // headers.insert("sec-fetch-dest", "empty".parse().unwrap());
        //     // headers.insert("sec-fetch-mode", "cors".parse().unwrap());
        //     // headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
        //     // headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36".parse().unwrap());
        //     // headers.insert("x-simulated-trading", "1".parse().unwrap());
        //     // headers.insert(header::COOKIE, "devId=f790c26b-e4c3-4383-90cd-0a1dabf7e4a2; locale=zh_CN; ok_prefer_currency=0%7C1%7Cfalse%7CUSD%7C2%7C%24%7C1%7C1%7C%E7%BE%8E%E5%85%83; fingerprint_id=f790c26b-e4c3-4383-90cd-0a1dabf7e4a2; first_ref=https%3A%2F%2Fwww.google.com%2F; ok_global={%22okg_m%22:%22xl%22}; okg.currentMedia=xl; g_state={\"i_l\":0,\"i_ll\":1763286138624,\"i_b\":\"6Xdw6ek/+lxvEkOsVP40+9SMBe5jI6FUVMPubsAHikw\"}; OptanonAlertBoxClosed=2025-11-16T09:42:19.891Z; _ym_uid=1763286144588938878; _ym_d=1763286144; _gcl_gs=2.1.k1$i1763286133$u112278938; _gcl_au=1.1.1032396099.1763286146; isLogin=1; _tk=fXKAKVrGee48G7rPP9WiYw==; ok_login_type=OKX_GLOBAL; ok_prefer_udColor=1; ok_prefer_udTimeZone=2; preferLocale=zh_CN; ok_site_info==0HNxojI5RXa05WZiwiIMFkQPx0Rfh1SPJiOiUGZvNmIsIyUVJiOi42bpdWZyJye; intercom-device-id-ny9cf50h=53dc7f5e-b80d-47a3-8838-29753b79f414; ok-exp-time=1764255857621; ok_prefer_cm=3; tmx_session_id=egwsozc8npl_1764255865581; fp_s=0; _gid=GA1.2.1252845773.1764255882; token=eyJraWQiOiIxMzYzODYiLCJhbGciOiJFUzI1NiJ9.eyJqdGkiOiJleDExMDE3NjMyODYyMTE5NjlFODY0MzZCQTlEQjUzNEQyMXNhQVQiLCJ1aWQiOiJQKzcvem1UWDFqdGpTa0IwUkkzOWJRPT0iLCJzdGEiOjAsIm1pZCI6IlArNy96bVRYMWp0alNrQjBSSTM5YlE9PSIsInBpZCI6IlBUeUE4VzA5ekZVSkJHSjZZUk5HWXc9PSIsIm5kZSI6MCwiaWF0IjoxNzY0MjU1ODg3LCJleHAiOjE3NjU0NjU0ODcsImJpZCI6OTk5OSwiZG9tIjoid3d3Lm9reC5jb20iLCJlaWQiOjE0LCJpc3MiOiJva2NvaW4iLCJkaWQiOiJHNGU5UXVjZFk1WmR4T3NpNkw4MXJ5YWRpWFpXSW5QTzZDc2xpYlh5K21nelRzVHdrMU9SbkZTV0JiSXNuZFVwIiwiZmlkIjoiRzRlOVF1Y2RZNVpkeE9zaTZMODFyeWFkaVhaV0luUE82Q3NsaWJYeSttZ3pUc1R3azFPUm5GU1dCYklzbmRVcCIsImxpZCI6IlArNy96bVRYMWp0alNrQjBSSTM5YlE9PSIsInVmYiI6IlBUeUE4VzA5ekZVSkJHSjZZUk5HWXc9PSIsInVwYiI6ImlCcmEyVmhOb2t5UmloeGlKLzN6RXc9PSIsImt5YyI6Mywia3lpIjoic1ZrUEh4ak1Hb2Fhc2o2Z3RXMVB4L3VrTDhybnFpUm9ncHpFek0vTzRvRk84cWtiMWJMdGFmMklSVUtrTDd4RTd5ZEYvWU5DR1FXLzV5aTRWQnpUM1E9PSIsImNwayI6ImhCdjNtSEZjb0lETG5TckZ6dEdTTlpMT29aU2s1bUE4SHBQcE9MOFE1TlhpRFpVaW9EWVpPeHNLMGY2MVBvaUZJb0R3WnJPVTJYQXo2aUZKWktlbVRIbzFtVGpzTmpSNk9TZHgwaWNrMlNhZnlwZmxxSFhJaWJFSWhkVmV5alZZWWRBSFFIbDRkQ1dESkFCc3MxSzM1TlJWTjdiOUtJOENIc1dzKzJBYWN6UT0iLCJ2ZXIiOjEsImNsdCI6MiwidXVkIjoiQ3RibXB5NjRpVzdaK3BEeitSWmJxSVlpdE5oZDJwUzVvWFFRbkQ5SkF2Yz0ifQ.lkVJRfin7PifLBnz2xh4v5QafCL391LhLazCa0R6Z3Rz1WB7Ve0cXLZFjuzteElWY4gTvdyTduRiyS01jCZxtQ; simulatedTrading=1; intercom-session-ny9cf50h=cnpmNzU2K1RzSnlLRTJUVzJXVDdjeFdLZlMza3hBaWViYkcyRFU1Um85UFFLWFVEd29hVTVzc2ZnOG5DQWE4RUsyQlExTGpOZFJ5STB5N213K2FwK2RPdytYZjQwdm1NMmtlSUZjU09vdWc9LS1ma0xsMlkrcHlvY2FkUzh2WkptS0F3PT0=--c776d35001e87d00449a93a08c9034c0b277b798; OptanonConsent=isGpcEnabled=0&datestamp=Thu+Nov+27+2025+23%3A34%3A25+GMT%2B0800+(%E4%B8%AD%E5%9B%BD%E6%A0%87%E5%87%86%E6%97%B6%E9%97%B4)&version=202405.1.0&browserGpcFlag=0&isIABGlobal=false&hosts=&landingPath=NotLandingPage&AwaitingReconsent=false&groups=C0004%3A1%2CC0002%3A1%2CC0003%3A1%2CC0001%3A1&geolocation=US%3BNY; _ga=GA1.1.1063686080.1763286146; _monitor_extras={\"deviceId\":\"tFdZDRBdiivnu3hsXFxYTL\",\"eventId\":95,\"sequenceNumber\":95}; _ga_G0EKWWQGTZ=GS2.1.s1764340964$o5$g0$t1764340964$j60$l0$h0; traceId=2140543409726690002; __cf_bm=gVKVQcSdWe8KGYiMCDGlYf5voqY.3njSVVFdQLz7XEM-1764340973-1.0.1.1-WFyIGeyF17hkDfcZMMNfhkyKEQyvfcPvFFsGeotlJx_SIQPWhpDc00cLv3JoF0j0mGA_lk4nSp5ksMrVP0jPjt59_TonGmbWfYgi5aoHmfM; ok-ses-id=IJ1q53AFsXwz+CwCC8rjbFtct2I3kzo+/l2IrGFGzdCn8R8jBFbE4gbFIyKlAflEDxcDqBDDVhRovTQZaVyg1ESFrMzgASHsidg2qKCePrbegKKZzNQFSKskleWl1Dn5".parse().unwrap());
        //     //
        //     // let client = reqwest::blocking::Client::builder()
        //     //     .redirect(reqwest::redirect::Policy::none())
        //     //     .build()
        //     //     .unwrap();
        //     // let res = client.get("https://www.okx.com/api/v5/account/balance?ccy=BTC")
        //     //     .headers(headers)
        //     //     .send()?
        //     //     .text()?;
        //     // sign("2025-11-28T14:43:34.282Z","GET","/api/v5/account/balance?ccy=BTC",)

        //     // Ok(())
        // }
    }
}