FROM aqua

# Set the noninteractive timezone (prevents configuration prompts)
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/London

# Set a working directory
WORKDIR /aqua/experiments/nexmark/

# Install apt packages
RUN apt-get update && apt-get install -y \
  curl wget software-properties-common default-jdk python3 python3-pip maven

# Install Apache Flink
RUN wget https://downloads.apache.org/flink/flink-1.18.1/flink-1.18.1-bin-scala_2.12.tgz \
  && tar xzf flink-1.18.1-bin-scala_2.12.tgz \
  && mv flink-1.18.1 /opt/flink
ENV PATH="/opt/flink/bin:${PATH}"

# Set the environment variable for JAVA_HOME
RUN JAVA_HOME="$(dirname $(dirname $(update-alternatives --list java | head -n 1)) | tail -1)"

# Install Python dependencies
RUN pip install matplotlib numpy

# Cleanup
RUN apt-get clean

# Copy the Flink and Rust source code into the image
COPY . .

# Copy Flink configuration
COPY flink-conf.yaml /opt/flink/conf/flink-conf.yaml

CMD ["python3", "run.py"]
