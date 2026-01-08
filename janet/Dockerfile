FROM leafgarland/janet-sdk
RUN mkdir /janet_src
COPY . /janet_src
RUN jpm install judge
USER nobody
ENTRYPOINT ["judge", "/janet_src"]
