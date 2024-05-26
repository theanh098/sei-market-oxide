  CREATE OR REPLACE VIEW "collection_view" AS
      SELECT 
      "c"."address", 
      "c"."name", 
      "c"."symbol", 
      "c"."supply", 
      "c"."royalty", 
      "c"."image", 
      "c"."banner", 
      "c"."description", 
      "c"."socials", 
      count("l"."id") "listed",
      coalesce(min("l"."price"),0) "floor_price",
      coalesce(max("l"."price"),0) "ceiling_price",
      (
          SELECT count("t"."id")
          FROM "public"."transaction" "t"
          WHERE "t"."collection_address" = "c"."address"
      ) "sales",
      (
          SELECT coalesce(sum("t"."volume"),0)
          FROM "public"."transaction" "t"
          WHERE "t"."collection_address" = "c"."address"
      ) "volume",
      (
          SELECT coalesce(sum("t"."volume"),0)
          FROM "public"."transaction" "t"
          WHERE "t"."collection_address" = "c"."address"
          AND "t"."date" > NOW() - INTERVAL '1 hour'
      ) "volume_of_1h",
      (
          SELECT coalesce(sum("t"."volume"),0)
          FROM "public"."transaction" "t"
          WHERE "t"."collection_address" = "c"."address"
          AND "t"."date" > NOW() - INTERVAL '1 day'
      ) "volume_of_24h",
      (
          SELECT coalesce(sum("t"."volume"),0)
          FROM "public"."transaction" "t"
          WHERE "t"."collection_address" = "c"."address"
          AND "t"."date" > NOW() - INTERVAL '7 days'
      ) "volume_of_7d",
      (
          SELECT coalesce(sum("t"."volume"),0)
          FROM "public"."transaction" "t"
          WHERE "t"."collection_address" = "c"."address"
          AND "t"."date" > NOW() - INTERVAL '30 days'
      ) "volume_of_30d"

      FROM "public"."collection" "c"
      LEFT JOIN "public"."nft" "n" ON "n"."token_address" = "c"."address"
      LEFT JOIN "public"."listing_nft" "l" 
          ON "l"."nft_id" = "n"."id" 
          AND ("l"."expiration_time" IS NULL OR "l"."expiration_time" > EXTRACT(epoch FROM NOW()))
      GROUP BY "c"."address";







     