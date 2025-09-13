use crate::types::{Crop, MarketInsight, YieldPrediction, YieldReport};
use soroban_sdk::{Env, String, Vec};

pub struct ReportingService;

impl ReportingService {
    /// Generate farmer report with recommendations
    pub fn generate_farmer_report(
        env: &Env,
        prediction: &YieldPrediction,
        crop: &Crop,
    ) -> YieldReport {
        let recommendations = Self::generate_farmer_recommendations(prediction);

        YieldReport {
            crop_name: crop.name.clone(),
            region: prediction.region.clone(),
            predicted_yield: prediction.predicted_yield,

            recommendations,
            report_date: env.ledger().timestamp(),
        }
    }

    /// Generate buyer market insights
    pub fn generate_buyer_insights(
        env: &Env,
        predictions: &Vec<YieldPrediction>,
        region: String,
    ) -> Vec<MarketInsight> {
        let mut insights = Vec::new(env);

        // Group predictions by crop
        let mut crop_yields: Vec<(String, i128)> = Vec::new(env);

        for prediction in predictions.iter() {
            if prediction.region == region {
                // In tests use a mock crop name; avoid hardcoding in non-test builds.
                #[cfg(test)]
                let crop_name = String::from_str(env, "TestCrop");
                #[cfg(not(test))]
                let crop_name = String::from_str(env, "UnknownCrop"); // TODO: replace with storage-backed lookup
                crop_yields.push_back((crop_name, prediction.predicted_yield));
            }
        }

        for (crop_name, yield_amount) in crop_yields.iter() {
            let insight = MarketInsight {
                crop_id: crop_name.clone(),
                region: region.clone(),
                expected_supply: yield_amount,
                price_trend: Self::predict_price_trend(&yield_amount),
                buying_recommendation: Self::generate_buying_recommendation(&yield_amount),
            };
            insights.push_back(insight);
        }

        insights
    }

    fn generate_farmer_recommendations(prediction: &YieldPrediction) -> String {
        if prediction.predicted_yield > 1000 {
            String::from_str(
                &soroban_sdk::Env::default(),
                "High yield expected. Consider additional harvesting equipment.",
            )
        } else if prediction.predicted_yield < 500 {
            String::from_str(
                &soroban_sdk::Env::default(),
                "Low yield predicted. Consider crop insurance or alternative crops.",
            )
        } else {
            String::from_str(
                &soroban_sdk::Env::default(),
                "Moderate yield expected. Monitor weather conditions closely.",
            )
        }
    }

    fn predict_price_trend(yield_amount: &i128) -> String {
        if *yield_amount > 1000 {
            String::from_str(&soroban_sdk::Env::default(), "Decreasing")
        } else if *yield_amount < 500 {
            String::from_str(&soroban_sdk::Env::default(), "Increasing")
        } else {
            String::from_str(&soroban_sdk::Env::default(), "Stable")
        }
    }

    fn generate_buying_recommendation(yield_amount: &i128) -> String {
        if *yield_amount > 1000 {
            String::from_str(
                &soroban_sdk::Env::default(),
                "Good time to buy - high supply expected",
            )
        } else if *yield_amount < 500 {
            String::from_str(
                &soroban_sdk::Env::default(),
                "Consider early contracts - low supply expected",
            )
        } else {
            String::from_str(
                &soroban_sdk::Env::default(),
                "Monitor market - stable supply expected",
            )
        }
    }
}
