FROM leafgarland/janet-sdk
RUN mkdir /janet
COPY . /janet
RUN jpm install judge
USER nobody
ENTRYPOINT ["judge", "/janet"]
