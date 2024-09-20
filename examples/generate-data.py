import csv
import random
from datetime import datetime, timedelta

num_points = 1_000_000

output_file = "data.csv"

start_time = datetime.now()


def random_noise():
    return timedelta(seconds=random.randint(-15, 15))


# Generate data points and write to CSV
with open(output_file, mode='w', newline='') as file:
    writer = csv.writer(file)

    for i in range(num_points):
        timestamp = start_time + timedelta(minutes=i) + random_noise()
        key = f"key_{random.randint(0, 99)}"
        value = random.randint(0, 999)

        writer.writerow([timestamp.isoformat(), key, value])

print(f"Data written to {output_file}")
