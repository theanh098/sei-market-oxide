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







       "SELECT "collection_view"."address", "collection_view"."name", "collection_view"."symbol", "collection_view"."image", "collection_view"."banner", "collection_view"."description", "collection_view"."royalty", "collection_view"."supply", "collection_view"."socials", "collection_view"."listed", "collection_view"."floor_price", "collection_view"."ceiling_price", "collection_view"."sales", "collection_view"."volume", "collection_view"."volume_of_1h", "collection_view"."volume_of_24h", "collection_view"."volume_of_30d" FROM "collection_view" WHERE "collection_view"."address" LIKE '%a%' LIMIT 11 OFFSET 0"