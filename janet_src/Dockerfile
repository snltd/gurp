FROM leafgarland/janet-sdk
RUN mkdir /janet_src
COPY . /janet_src
RUN jpm install judge
ENTRYPOINT ["judge", "/janet_src"]
