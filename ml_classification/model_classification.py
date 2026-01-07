import psycopg2
import pandas as pd
import numpy as np
from sklearn.cluster import KMeans
from datetime import datetime


DB_URL = "postgresql://postgres:root@localhost:5434/sedentary_data"

# calibrated thresholds
THRESH_FIDGET = 0.020
THRESH_ACTIVE = 0.040

def run_analysis():
    print(" Starting Nightly Model Analysis ...")

    # 1. CONNECT TO DATABASE (Instead of Serial)
    try:
        conn = psycopg2.connect(DB_URL)
        
        # Read raw data from the last 24 hours 
        # We fetch the smoothed 'acceleration_val' 
        query = """
            SELECT created_at, acceleration_val, state 
            FROM sedentary_log 
            WHERE created_at > NOW() - INTERVAL '24 HOURS'
            ORDER BY created_at ASC
        """
        df = pd.read_sql(query, conn)
        
        if df.empty:
            print(" No data found for today.")
            return

        print(f"Loaded {len(df)} data points from PostgreSQL.")

        # 2. APPLY YOUR MODEL LOGIC
        # We can re-verify the classification or calculate stats
        
        # Calculate Logic Stats
        active_count = len(df[df['acceleration_val'] > THRESH_ACTIVE])
        fidget_count = len(df[(df['acceleration_val'] > THRESH_FIDGET) & (df['acceleration_val'] <= THRESH_ACTIVE)])
        sedentary_count = len(df[df['acceleration_val'] <= THRESH_FIDGET])

        # 3. ADVANCED ANALYSIS (KMeans)
        # We use KMeans logic to see if new behaviors emerged today
        if len(df) > 100:
            X = df[['acceleration_val']]
            kmeans = KMeans(n_clusters=3, random_state=42, n_init=10)
            df['cluster'] = kmeans.fit_predict(X)
            
            # Identify the Deep Sedentary cluster center
            centers = sorted(kmeans.cluster_centers_.flatten())
            print(f"🔎 Today's Detected Clusters: {centers}")
            
            # Adaptive Threshold Suggestion
            # If the lowest cluster center shifted, then we need to update thresholds
            suggested_fidget_threshold = (centers[0] + centers[1]) / 2
            print(f"💡 Suggested New Fidget Threshold: {suggested_fidget_threshold:.4f}")

        # 4. SAVE RESULTS (For the History Graph)
        # Convert counts to minutes 
        sedentary_mins = round(sedentary_count / 600, 2)
        active_mins = round((active_count + fidget_count) / 600, 2)
        
        total_mins = sedentary_mins + active_mins
        score = int((active_mins / total_mins) * 100) if total_mins > 0 else 0
        
        dominant_state = "SEDENTARY"
        if active_mins > sedentary_mins: dominant_state = "ACTIVE"

        cursor = conn.cursor()
        cursor.execute("""
            INSERT INTO activity_summary (date, sedentary_minutes, active_minutes, dominant_state, activity_score)
            VALUES (CURRENT_DATE, %s, %s, %s, %s)
            ON CONFLICT (date) DO UPDATE 
            SET sedentary_minutes = EXCLUDED.sedentary_minutes,
                active_minutes = EXCLUDED.active_minutes,
                activity_score = EXCLUDED.activity_score;
        """, (sedentary_mins, active_mins, dominant_state, score))
        
        conn.commit()
        print(f"Analysis Saved! Score: {score}/100")
        
    except Exception as e:
        print(f" Error: {e}")
    finally:
        if conn: conn.close()

if __name__ == "__main__":
    run_analysis()