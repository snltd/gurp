FROM leafgarland/janet-sdk:v1.41.1
RUN mkdir /janet
COPY . /janet
RUN jpm install judge
USER nobody
ENTRYPOINT ["judge", "/janet/test"]
